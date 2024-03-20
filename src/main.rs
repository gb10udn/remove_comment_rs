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

    let mut temp_dst = PathBuf::from(r".\dst_rmc");
    let now: String = Local::now()
        .format("%Y%m%d_%H%M%S")
        .to_string();  // HACK: 240320 次とのつながり (Ex. ビルド) を考えると、日付情報を入れるのは良くない気がしてきた。
    temp_dst.push(&now);

    let folder_name = Path::new(&src).file_name().unwrap();  // HACK: 240320 folder_name を削除を検討する。(file or dir での操作統一のため)
    temp_dst.push(folder_name);

    let dst_base_dir = temp_dst.to_string_lossy().to_string();

    let path_vec = retrieve_path_vec(&src, &config.target_extensions);
    let mut error_messages: Vec<String> = vec![];
    for fpath in path_vec {
        let dst = fpath.replace(&src, &dst_base_dir);
        if let Err(err) = try_to_remove_comment_and_save_one(&fpath, &dst, &config.remove_comments, &config.target_extensions, &config.remove_multiline_comment) {
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
    // TODO: 240220 フルパスでどこのファイルを処理したかは、別途 log ファイルに残してあげるといいような気もする。(.exe ダブルクリックで実施するなら、必須かもしれない？)
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
                    let _ = Command::new("./vba.exe")
                    .args([
                        "--src",
                        src as &str,
                        "--dst",
                        dst.to_str().unwrap(),
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())  // FIXME: 240313 エラー処理を記述する。(現状では、何も知らせないため不親切な仕様になってしまっている。)
                    .spawn()
                    .unwrap();

                    Ok(())
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

/// src がディレクトリの場合、再帰的に検索したパスのベクタ型を、ファイルパスの場合、その要素を持ったベクタ型を返す関数。
fn retrieve_path_vec(src: &String, target_extensions: &Vec<String>) -> Vec<String> {
    let src = remove_head_and_tail_double_quotation(src);
    let src = Path::new(&src);

    let mut result: Vec<String> = vec![];
    if src.is_file() {
        result.push(src.to_string_lossy().to_string())
    
    } else {
        for entry in WalkDir::new(src) {
            if let Ok(val) = entry {
                if val.path().is_file() {
                    if let Some(ext) = val.path().extension() {
                        let ext = ext.to_str().unwrap();
                        if target_extensions.contains(&ext.to_string()) {
                            let fpath = val.path().to_string_lossy().to_string();
                            result.push(fpath);
                        }
                    }
                }
            }
        }
    }
    result
}

fn remove_head_and_tail_double_quotation(arg: &String) -> String {
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
    fn test_retrieve_path_vec() {
        use crate::retrieve_path_vec;

        let src = r".\misc";
        let target_extensions = vec![String::from("py")];
        assert_eq!(retrieve_path_vec(&src.to_string(), &target_extensions), vec![String::from(r".\misc\piyo\sample_002.py"), String::from(r".\misc\sample_001.py"),]);
        
        let target_extensions = vec![String::from("ps")];  // INFO: 240221 ps を指定すると、py ファイルは取得しない。
        assert_ne!(retrieve_path_vec(&src.to_string(), &target_extensions), vec![String::from(r".\misc\piyo\sample_002.py"), String::from(r".\misc\sample_001.py"),]);
    }
}