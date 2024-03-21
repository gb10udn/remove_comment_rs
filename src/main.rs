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


fn main() -> Result<(), Box<dyn std::error::Error>> {  // TODO: 240228 result 返すのでなくて、結果を表示する方が、ユーザーにとって優しいかもしれない？
    let args = Args::parse();
    let src = args.src.expect("\n\nArgError: src must be given ...\n\n");
    let config = open_config("./config.json")?;

    let now = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let transfer_info_vec = retrieve_transfer_info_vec(&src, &now, &config.target_extensions);
    let mut error_messages: Vec<String> = vec![];
    for transfer_info in transfer_info_vec {
        if let Err(err) = try_to_remove_comment_and_save_one(&transfer_info.src, &transfer_info.dst, &config.remove_comments, &config.target_extensions, &config.remove_multiline_comment) {
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


// HACK: 240313 コンパクトにせよ。拡張子分岐のところが長いので、別関数にくくりだすとか？
fn try_to_remove_comment_and_save_one(src: &String, dst: &String, remove_comments: &Vec<String>, target_extensions: &Vec<String>, remove_multiline_comment: &bool) -> Result<(), Box<dyn std::error::Error>> {
    let src_ = Path::new(src);
    if let Some(ext) = src_.extension() {
        let ext = ext.to_str().unwrap();
        if target_extensions.contains(&ext.to_string()) {  // HACK: 240228 target ってのが、コメント削除のことか、コピー対象のことかが分かりにくい。target_text_file_extensions の方が長いけどわかりやすいかも？
            
            // [START] create dist basedir
            let dst = Path::new(dst);
            let base_path = dst
                .parent()
                .unwrap();
            fs::create_dir_all(base_path).unwrap();
            // [END] create dist basedir
            
            match ext {
                "xlsm" => {
                    let output = Command::new("./vba.exe")
                        .args([
                            "--src",
                            src as &str,
                            "--dst",
                            dst.to_str().unwrap(),
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
                _ => {
                    if let Ok(mut code) = opf::text::open_file(&src) {
                        match ext {
                            "py" => {
                                if *remove_multiline_comment {
                                    code = rmc::py::remove_multiline_comment(&code);
                                }
                                code = rmc::py::remove_comment(&code, &remove_comments);
                            }
                            "ps1" => {
                                if *remove_multiline_comment {
                                    code = rmc::ps::remove_multiline_comment(&code);
                                }
                                code = rmc::ps::remove_comment(&code, &remove_comments);
                            }
                            "psd1" => {
                                if *remove_multiline_comment {
                                    code = rmc::ps::remove_multiline_comment(&code);
                                }
                                code = rmc::ps::remove_comment(&code, &remove_comments);
                            }
                            "psm1" => {
                                if *remove_multiline_comment {
                                    code = rmc::ps::remove_multiline_comment(&code);
                                }
                                code = rmc::ps::remove_comment(&code, &remove_comments);
                            }
                            _ => {
                                // INFO: 240310 .json など、コピペするだけのファイル。  // FIXME: 240310 この場合、わざわざテキストファイルとして開く必要がない気がしてきた。エラーの温床だし。
                            }
                        }
                        
                        // [START] save text file
                        let mut file = File::create(dst)
                            .expect("file not found.");
                    
                        write!(file, "{}", code)
                            .expect("cannot write.");
                        // [END] save text file
        
                        Ok(())
        
                    } else {
                        Err(format!("Fail to open file -> {}", src).into())
                    }
                },
            }
        } else {
            Err(format!("").into())  // FIXME: 240228 ここの運用が少し微妙かも？taraget_ext でないファイルパスを渡すな、と。
        }
    } else {
        Err(format!("No Extension ? {}", src).into())  // INFO: 240228 拡張子を持たないテキストファイルは対象としない前提とした。
    }
}

#[derive(Debug, PartialEq)]
struct TransferInfo {
    src: String,
    dst: String,
}

/// src がディレクトリの場合、再帰的に検索したパスのベクタ型を、ファイルパスの場合、その要素を持ったベクタ型を返す関数。
fn retrieve_transfer_info_vec(src: &String, folder_name: &String, target_extensions: &Vec<String>) -> Vec<TransferInfo> {
    let src = remove_head_and_tail_double_quotation(src);
    let src = Path::new(&src);

    let mut temp_dst = PathBuf::from(r".\dst_rmc");
    temp_dst.push(folder_name);

    let mut result: Vec<TransferInfo> = vec![];
    if src.is_file() {
        if let Some(ext) = src.extension() {
            if target_extensions.contains(&ext.to_string_lossy().to_string()) {
                let fname = src.file_name().unwrap().to_str().unwrap();
                temp_dst.push(fname);
                result.push(TransferInfo { src: src.to_string_lossy().to_string(), dst: temp_dst.to_str().unwrap().to_string()});
            }
        }
    } else {
        let dst_base_dir = temp_dst.to_string_lossy().to_string();
        for entry in WalkDir::new(src) {
            if let Ok(val) = entry {
                if val.path().is_file() {
                    if let Some(ext) = val.path().extension() {
                        let ext = ext.to_str().unwrap();
                        if target_extensions.contains(&ext.to_string()) {
                            let fpath = val.path().to_string_lossy().to_string();
                            let dst = fpath.replace(src.to_str().unwrap(), &dst_base_dir);
                            let dst = Path::new(&dst);
                            result.push(TransferInfo { src: fpath, dst: dst.to_string_lossy().to_string()});
                        }
                    }
                }
            }
        }
    }
    result
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
    target_extensions: Vec<String>,
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

        let src = r".\misc";
        let target_extensions = vec![String::from("py")];
        let expected = vec![
            TransferInfo { src: String::from(r".\misc\piyo\sample_002.py"), dst: String::from(r".\dst_rmc\test\piyo\sample_002.py") },
            TransferInfo { src: String::from(r".\misc\sample_001.py"),      dst: String::from(r".\dst_rmc\test\sample_001.py") },
        ];
        let result = retrieve_transfer_info_vec(&src.to_string(), &String::from("test"), &target_extensions);
        assert_eq!(result, expected);
    }
}