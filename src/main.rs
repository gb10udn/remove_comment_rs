use std::io::{Write, BufReader};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use chrono::Local;
use walkdir::WalkDir;
use clap::Parser;
use serde::{Deserialize, Serialize};

mod rmc;
mod opf;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let src = args.src.expect("\n\nArgError: src must be given ...\n\n");
    let config = open_config("./config.json")?;

    let now = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let transfer_info_vec = retrieve_transfer_info_vec(&src, &now, &config.copy_extensions);
    let mut error_messages: Vec<String> = vec![];
    for transfer_info in transfer_info_vec {
        if let Err(err) = remove_comment_and_save(&transfer_info,  &config.remove_comments, &config.remove_multiline_comment) {
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


fn remove_comment_and_save(transfer_info: &TransferInfo, remove_comments: &Vec<String>, remove_multiline_comment: &bool) -> Result<(), Box<dyn std::error::Error>> {
    if transfer_info.proc_type != ProcType::Skip {
        if let Some(base_path) = Path::new(&transfer_info.dst).parent() {
            fs::create_dir_all(base_path).unwrap();
        }
    }

    match transfer_info.proc_type {
        ProcType::Xlsm => {
            let output = Command::new("./vba.exe")
                .args([
                    "--src",
                    &transfer_info.src as &str,
                    "--dst",
                    transfer_info.dst.as_str(),
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
        },
        ProcType::Py => {
            let mut code = opf::text::open_file(&transfer_info.src).expect("");
            if *remove_multiline_comment {
                code = rmc::py::remove_multiline_comment(&code);
            }
            code = rmc::py::remove_comment(&code, &remove_comments);
            let mut file = File::create(&transfer_info.dst).expect("file not found.");
            write!(file, "{}", code).expect("cannot write.");
            Ok(())
        },
        ProcType::Ps => {
            let mut code = opf::text::open_file(&transfer_info.src).expect("");
            if *remove_multiline_comment {
                code = rmc::ps::remove_multiline_comment(&code);
            }
            code = rmc::ps::remove_comment(&code, &remove_comments);
            let mut file = File::create(&transfer_info.dst).expect("file not found.");
            write!(file, "{}", code).expect("cannot write.");
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
fn retrieve_transfer_info_vec(src: &String, folder_name: &String, copy_extensions: &Vec<String>) -> Vec<TransferInfo> {
    let src = remove_head_and_tail_double_quotation(src);
    let src = Path::new(&src);

    let mut temp_dst = PathBuf::from(r".\dst_rmc");
    temp_dst.push(folder_name);

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
        for entry in WalkDir::new(src) {
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


fn remove_head_and_tail_double_quotation(arg: &String) -> String {  // FIXME: 240320 使用しない可能性が高まったので、削除してよい。
    let mut result = arg.clone();
    if result.ends_with("\n") == true {
        result.pop();  // INFO: 240113 標準入力で取得時の末尾の改行コードを除去するため。
    }
    if result.starts_with("\"") {
        result.remove(0);
    }
    if result.ends_with("\"") {
        result.pop();
    }
    result
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    remove_multiline_comment: bool,
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
    fn test_remove_head_and_tail_double_quotation() { 
        use crate::remove_head_and_tail_double_quotation;

        assert_eq!(remove_head_and_tail_double_quotation(&String::from("abc\n")), String::from("abc"));
        assert_eq!(remove_head_and_tail_double_quotation(&String::from("\"abc\"\n")), String::from("abc"));
    }

    #[test]
    fn test_retrieve_transfer_info_vec() {
        use crate::retrieve_transfer_info_vec;
        use crate::TransferInfo;
        use crate::ProcType;

        let src = r".\misc";
        let copy_extensions = vec![String::from("py")];
        let expected = vec![
            TransferInfo { src: String::from(r".\misc\piyo\sample_002.py"), dst: String::from(r".\dst_rmc\test\piyo\sample_002.py"), proc_type: ProcType::Py },
            TransferInfo { src: String::from(r".\misc\sample_001.py"),      dst: String::from(r".\dst_rmc\test\sample_001.py")     , proc_type: ProcType::Py },
        ];
        let temp_result = retrieve_transfer_info_vec(&src.to_string(), &String::from("test"), &copy_extensions);
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