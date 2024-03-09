use std::io::Write;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use chrono::Local;
use walkdir::WalkDir;
use clap::Parser;
mod rmc;
mod opf;


fn main() -> Result<(), Box<dyn std::error::Error>> {  // TODO: 240228 result 返すのでなくて、結果を表示する方が、ユーザーにとって優しいかもしれない？
    // [START] set up params  // HACK: 240228 拡張子とかの設定は、config ファイルで運用するのが楽なような気もする。
    let args = Args::parse();  // HACK: 240221 引数の渡し方は運用決めてから再度検討すること。
    let rm_multiline_comment = true;
    let remove_comments = vec!["TODO:", "FIXME:", "EDIT:", "HACK:", "INFO:", "[START]", "[END]"];
    let target_extensions = vec!["py", "ps1", "psd1", "psm1", "xlsm", "txt", "json"];  // HACK: 240228 target の意味が分かりにくい。copy する text ファイルってことが分かりにくい。
    // [END] set up params


    // [START] set up src / src_ params
    let src: String;
    match args.src {
        Some(val) => {
            src = val;
        }
        None => {
            src = fs::canonicalize("./").unwrap().to_string_lossy().to_string();
        }
    }
    let src = remove_head_and_tail_double_quotation(&src);  // HACK: 240219 タイミングは要検討 (対話的にユーザー入力を取得しない限りは不要かも？)
    let src_ = Path::new(&src);
    // [END] set up src / src_ params

    
    let mut temp_dst = PathBuf::from(r".\dst_rmc");
    let now: String = Local::now()
        .format("%Y%m%d_%H%M%S")
        .to_string();
    temp_dst.push(&now);
    
    if src_.is_file() {
        temp_dst.push(src_.file_name().unwrap());
        let dst = temp_dst.to_string_lossy().to_string();
        match try_to_remove_comment_and_save_one(&src, &dst, &remove_comments, &target_extensions, &rm_multiline_comment) {
            Ok(_) => {
                Ok(())
            },
            Err(err) => {
                Err(err.into())
            }
        }

    } else if src_.is_dir() {
        let folder_name = src_.file_name().unwrap();
        temp_dst.push(folder_name);
        let dst_base_dir = temp_dst.to_string_lossy().to_string();
        
        let path_vec = retrieve_path_vec(&src, &target_extensions);
        let mut error_messages: Vec<String> = vec![];
        for fpath in path_vec {  // TODO: 240228 最後に、ファイル何個が存在し、ターゲットのテキストファイルが何件で、処理したのが何件で、、、を表示するといいかも？
            let dst = fpath.replace(&src, &dst_base_dir);
            if let Err(err) = try_to_remove_comment_and_save_one(&fpath, &dst, &remove_comments, &target_extensions, &rm_multiline_comment) {
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
    } else {
        Err(format!("File Not Found -> {}", src).into())
    }

    // TODO: 240220 requirement.txt があり、pyinsraller が存在する場合は、ビルドまでやってあげる？
    // TODO: 240220 フルパスでどこのファイルを処理したかは、別途 log ファイルに残してあげるといいような気もする。(.exe ダブルクリックで実施するなら、必須かもしれない？)
}


fn try_to_remove_comment_and_save_one(src: &String, dst: &String, remove_comments: &Vec<&str>, target_extensions: &Vec<&str>, rm_multiline_comment: &bool) -> Result<(), Box<dyn std::error::Error>> {
    let src_ = Path::new(src);
    if let Some(ext) = src_.extension() {
        let ext = ext.to_str().unwrap();
        if target_extensions.contains(&ext) {  // HACK: 240228 target ってのが、コメント削除のことか、コピー対象のことかが分かりにくい。target_text_file_extensions の方が長いけどわかりやすいかも？
            if let Ok(mut code) = opf::open_file(&src) {
                match ext {
                    "py" => {
                        if *rm_multiline_comment {
                            code = rmc::py::remove_multiline_comment(&code);
                        }
                        code = rmc::py::remove_comment(&code, &remove_comments);
                    }
                    "ps1" => {
                        if *rm_multiline_comment {
                            code = rmc::ps::remove_multiline_comment(&code);
                        }
                        code = rmc::ps::remove_comment(&code, &remove_comments);
                    }
                    "psd1" => {
                        if *rm_multiline_comment {
                            code = rmc::ps::remove_multiline_comment(&code);
                        }
                        code = rmc::ps::remove_comment(&code, &remove_comments);
                    }
                    "psm1" => {
                        if *rm_multiline_comment {
                            code = rmc::ps::remove_multiline_comment(&code);
                        }
                        code = rmc::ps::remove_comment(&code, &remove_comments);
                    }
                    "xlsm" => {
                        // TODO: 240220 (将来用) xlsm (バイナリファイルで特殊だから分けた方がいいかも？)
                    }
                    _ => {}
                }
                // [START] create dist basedir
                let dst = Path::new(dst);
                let base_path = dst
                    .parent()
                    .unwrap();
                fs::create_dir_all(base_path).unwrap();  // HACK: 240218 (あまり考えられないが) 重複したフォルダを操作する場合に処理止めていいかも？
                // [END] create dist basedir
                
                // [START] save text file
                let mut file = File::create(dst)
                    .expect("file not found.");
            
                write!(file, "{}", code)
                    .expect("cannot write.");
                // [END] save text file

                Ok(())

            } else {
                Err(format!("Fail to open file -> {}", src).into())  // FIXME: 240228 utf-8, utf-16le 以外だとここに来てしまう (Ex. Shift-JIS) のを修正せよ。
            }
        } else {
            Err(format!("").into())  // FIXME: 240228 ここの運用が少し微妙かも？taraget_ext でないファイルパスを渡すな、と。
        }
    } else {
        Err(format!("No Extension -? {}", src).into())  // INFO: 240228 拡張子を持たないテキストファイルは対象としない前提とした。
    }
}

/// base_dir 配下のファイルを再帰的に検索し、そのパスのベクタ型を返す関数。
fn retrieve_path_vec(base_dir: &String, target_extensions: &Vec<&str>) -> Vec<String> {  // HACK: 240220 引数は、Path で与えてもいいのかも？
    let mut result: Vec<String> = vec![];
    for entry in WalkDir::new(base_dir) {
        if let Ok(val) = entry {
            if val.path().is_file() {
                if let Some(ext) = val.path().extension() {
                    let ext = ext.to_str().unwrap();
                    if target_extensions.contains(&ext) {
                        let fpath = val.path().to_string_lossy().to_string();
                        result.push(fpath);
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
        let target_extensions = vec!["py"];
        assert_eq!(retrieve_path_vec(&src.to_string(), &target_extensions), vec![String::from(r".\misc\piyo\sample_002.py"), String::from(r".\misc\sample_001.py"),]);
        
        let target_extensions = vec!["ps"];  // INFO: 240221 ps を指定すると、py ファイルは取得しない。
        assert_ne!(retrieve_path_vec(&src.to_string(), &target_extensions), vec![String::from(r".\misc\piyo\sample_002.py"), String::from(r".\misc\sample_001.py"),]);
    }
}