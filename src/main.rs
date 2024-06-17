use std::io::{Write, BufReader};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use walkdir::{WalkDir, DirEntry};
use clap::Parser;
use serde::{Deserialize, Serialize};

mod rmc;
mod opf;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let src = args.src.expect("\n\nArgError: src must be given ...\n\n");
    let config = open_config("./config.json")?;

    if Path::new(&config.dst).exists() == true {
        panic!("\n\n{}\n\n", format!("❌ Error: already existed -> {:?}. Try again after removing this dir (to avoid to overwrite)", &config.dst));
    }

    let transfer_info_vec = retrieve_transfer_info_vec(&src, &config.dst, &config.copy_extensions);
    let mut error_messages: Vec<String> = vec![];
    for transfer_info in transfer_info_vec {  // TODO: 240324 並行 or 並列処理にする。(エラー取得も検討せよ)
        if let Err(err) = remove_comment_and_save(&transfer_info,  &config.remove_comments, &config.remove_multiline_comment, &config.remove_excel_macro_test_code) {
            error_messages.push(err.to_string());
        }
    }
    match error_messages.len() {
        0 => {
            Ok(())
        },
        _ => {
            Err(format!("{:?}", error_messages).into())
        },
    }
}


fn remove_comment_and_save(transfer_info: &TransferInfo, remove_comments: &Vec<String>, remove_multiline_comment: &bool, remove_excel_macro_test_code: &bool) -> Result<(), Box<dyn std::error::Error>> {
    if transfer_info.proc_type != ProcType::Skip {
        if let Some(base_path) = Path::new(&transfer_info.dst).parent() {
            fs::create_dir_all(base_path).unwrap();
        }
    }

    match transfer_info.proc_type {
        ProcType::Xlsm => {
            const VBA_PATH: &str = "./vba.exe";
            if Path::is_file(Path::new(VBA_PATH)) == true {
                let output = Command::new(VBA_PATH)
                    .args([
                        "--src",
                        &transfer_info.src as &str,
                        "--dst",
                        transfer_info.dst.as_str(),
                        "--remove-multiline-comment",
                        convert_bool_to_str(remove_multiline_comment),
                        "--remove-excel-macro-test-code",
                        convert_bool_to_str(remove_excel_macro_test_code),
                        
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .expect("\n\nFailed to execute command\n\n");

                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr != "" {
                    Err(stderr.into())
                } else {
                    Ok(())
                }
            } else {
                Err(format!("Error: File NOT FOUND -> {}", VBA_PATH).into())
            }
        },
        ProcType::Py => {
            let mut code = opf::text::open_file(&transfer_info.src)?;
            if remove_multiline_comment == &true {
                code = rmc::py::remove_multiline_comment(&code);
            }
            code = rmc::py::remove_comment(&code, &remove_comments);
            let mut file = File::create(&transfer_info.dst)?;
            write!(file, "{}", code)?;
            Ok(())
        },
        ProcType::Ps => {
            let mut code = opf::text::open_file(&transfer_info.src)?;
            if remove_multiline_comment == &true {
                code = rmc::ps::remove_multiline_comment(&code);
            }
            code = rmc::ps::remove_comment(&code, &remove_comments);
            let mut file = File::create(&transfer_info.dst)?;
            write!(file, "{}", code)?;
            Ok(())
        }
        ProcType::Copy => {
            fs::copy(&transfer_info.src, &transfer_info.dst)?;
            Ok(())
        }
        ProcType::Skip => {
            Ok(())
        }
    }
}


#[derive(Debug, PartialEq)]
struct TransferInfo {
    src: String,
    dst: String,
    proc_type: ProcType,
}

#[derive(Debug, PartialEq)]
enum ProcType {
    Py,
    Ps,
    Xlsm,
    Copy,
    Skip,
}

/// src がディレクトリの場合は再帰的に検索、ファイルパスの場合はその値を持った TransferInfo のベクタ型を返す関数。
fn retrieve_transfer_info_vec(src: &String, dst_dir: &String, copy_extensions: &Vec<String>) -> Vec<TransferInfo> {  // TODO: 240413 除外するパスを指定するといいかも？
    let src = Path::new(&src);
    let mut temp_dst = PathBuf::from(dst_dir);

    let mut result: Vec<TransferInfo> = vec![];
    if src.is_file() {
        let fname = src.file_name().unwrap().to_str().unwrap();
        temp_dst.push(fname);
        result.push(TransferInfo { 
            src: src.to_string_lossy().to_string(),
            dst: temp_dst.to_str().unwrap().to_string(),
            proc_type: obtain_proc_type(src, copy_extensions),
        });
    } else {
        let dst_base_dir = temp_dst.to_string_lossy().to_string();
        for entry in WalkDir::new(src).into_iter().filter_entry(|f| !is_hidden(f)) {
            if let Ok(val) = entry {
                if val.path().is_file() {
                    let fpath = val.path().to_string_lossy().to_string();
                    let dst = fpath.replace(src.to_str().unwrap(), &dst_base_dir);
                    let dst = Path::new(&dst);
                    result.push(TransferInfo {
                        src: fpath,
                        dst: dst.to_string_lossy().to_string(),
                        proc_type: obtain_proc_type(val.path(), copy_extensions)
                    });
                }
            }
        }
    }
    result
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}


fn obtain_proc_type(path: &Path, copy_extensions: &Vec<String>) -> ProcType {
    if let Some(ext) = path.extension() {
        match ext.to_str().expect("Fail to obtain extension ...") {
            "xlsm" => ProcType::Xlsm,
            "ps1" => ProcType::Ps,
            "psd1" => ProcType::Ps,
            "psm1" => ProcType::Ps,
            "py" => ProcType::Py,
            _ => {
                if copy_extensions.contains(&ext.to_string_lossy().to_string()) {
                    ProcType::Copy
                } else {
                    ProcType::Skip
                }
            }
        }
    } else {
        ProcType::Skip
    }
}

fn convert_bool_to_str(arg: &bool) -> &str {
    if arg == &true {
        "1"
    } else {
        "0"
    }
}


#[derive(Serialize, Deserialize, Debug)]
struct Config {
    dst: String,
    remove_multiline_comment: bool,
    remove_excel_macro_test_code: bool,
    remove_comments: Vec<String>,
    copy_extensions: Vec<String>,
}


fn open_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)?;
    Ok(config)
}


#[derive(Parser, Debug)]
#[command()]
struct Args {
    /// コメント削除する source のパス。ファイル or ディレクトリを指定する。
    #[arg(short = 's', long)]
    src: Option<String>,
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_retrieve_transfer_info_vec() {
        use crate::retrieve_transfer_info_vec;
        use crate::TransferInfo;
        use crate::ProcType;

        let src = r".\misc";
        let copy_extensions = vec![String::from("py")];
        let expected = vec![
            TransferInfo { src: String::from(r".\misc\piyo\sample_002.py"), dst: String::from(r".\dst_rmc\piyo\sample_002.py"), proc_type: ProcType::Py },
            TransferInfo { src: String::from(r".\misc\sample_001.py"),      dst: String::from(r".\dst_rmc\sample_001.py")     , proc_type: ProcType::Py },
        ];
        let temp_result = retrieve_transfer_info_vec(&src.to_string(), &String::from(r".\dst_rmc"), &copy_extensions);
        let mut result = vec![];
        for res in temp_result {
            if res.proc_type == ProcType::Py {
                result.push(res);
            }
        }
        assert_eq!(result, expected);
    }

    #[test]
    fn test_obtain_proc_type() {
        use crate::obtain_proc_type;
        use crate::ProcType;
        use std::path::Path;

        let result = obtain_proc_type(Path::new("hoge.py"), &vec![]);
        assert_eq!(result, ProcType::Py)
    }
}