use std::io::{prelude::*, Write};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use chrono::Local;
use walkdir::WalkDir;
use clap::Parser;
mod rmc;


fn main() {
    // [START] set up params
    let args = Args::parse();
    let src = args.src;
    let rm_multiline_comment = true;
    let targets = vec!["TODO:", "FIXME:", "EDIT:", "HACK:", "INFO:", "[START]", "[END]"];
    // [END] set up params

    let src = String::from(&src);
    let src = remove_head_and_tail_double_quotation(&src);  // HACK: 240219 タイミングは要検討 (対話的にユーザー入力を取得しない限りは不要かも？)
    let src_ = Path::new(&src);

    let mut temp_dst = PathBuf::from(r".\dst_rmc");
    let now: String = Local::now()
        .format("%Y%m%d_%H%M%S")
        .to_string();
    temp_dst.push(&now);
    
    if src_.is_file() {
        temp_dst.push(src_.file_name().unwrap());
        let dst = temp_dst.to_string_lossy().to_string();
        try_to_remove_comment_and_save_one(&src, &dst, &targets, &rm_multiline_comment);

    } else if src_.is_dir() {
        let folder_name = src_.file_name().unwrap();
        temp_dst.push(folder_name);
        let dst_base_dir = temp_dst.to_string_lossy().to_string();
        
        let path_vec = retrieve_path_vec(&src);
        for fpath in path_vec {
            let dst = fpath.replace(&src, &dst_base_dir);
            try_to_remove_comment_and_save_one(&fpath, &dst, &targets, &rm_multiline_comment);
        }

    } else {
        panic!("{}", format!("*****\nFetalError: unknown type of error  -> \"{}\"\n*****", src));
    }

    // TODO: 240220 requirement.txt があり、pyinsraller が存在する場合は、ビルドまでやってあげる？
    // TODO: 240220 フルパスでどこのファイルを処理したかは、別途 log ファイルに残してあげるといいような気もする。
}

fn try_to_remove_comment_and_save_one(src: &String, dst: &String, targets: &Vec<&str>, rm_multiline_comment: &bool) {
    let target_extensions = vec!["py", "ps", "xlsm", "txt", "json"];  // INFO: 240220 これ以外の拡張子の場合、保存もコメント削除も何も実行しない。
    let src_ = Path::new(src);
    if let Some(ext) = src_.extension() {
        let ext = ext.to_str().unwrap();
        if target_extensions.contains(&ext) {
            let mut code = open_file(&src);
            match ext {
                "py" => {
                    code = rmc::py::remove_comment(&code, &targets);
                    if *rm_multiline_comment {
                        code = rmc::py::remove_multiline_comment(&code);
                    }
                }
                "ps" => {
                    // TODO: 240220 (将来用) ps
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
            
            let mut file = File::create(dst)
                .expect("file not found.");  
        
            write!(file, "{}", code)
                .expect("cannot write.");
        }
    }
}

/// base_dir 配下のファイルを再帰的に検索し、そのパスのベクタ型を返す関数。
fn retrieve_path_vec(base_dir: &String) -> Vec<String> {  // HACK: 240220 引数は、Path で与えてもいいのかも？
    let mut result: Vec<String> = vec![];
    for entry in WalkDir::new(base_dir) {
        if let Ok(val) = entry {
            if val.path().is_file() {
                let fpath = val.path().to_string_lossy().to_string();
                result.push(fpath);
            }
        }
    }
    result
}

fn open_file(path: &String) -> String {
    let mut f = File::open(path).unwrap();  // FIXME: 240220 バイナリの場合失敗すると思うので、Option<String> を返した方がいいのかも？
    let mut result = String::new();
    f.read_to_string(&mut result).unwrap();
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
    src: String,
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
        assert_eq!(retrieve_path_vec(&src.to_string()), vec![String::from(r".\misc\piyo\sample_002.py"), String::from(r".\misc\sample_001.py"),]);
    }
}