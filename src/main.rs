use std::io::{prelude::*, Write};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use chrono::Local;
use walkdir::WalkDir;
mod rmc;


fn main() {
    // [START] set up params
    // let src = "./misc/sample_001.py";  // TODO: 240218 ここをユーザー入力 or プログラムの置かれたカレントディレクトリにする。
    // let src = r"C:\Users\t.imaishi\Documents\_imis\01_program\01_project\240213_remove_comment_rs\remove_comment_rs\misc";
    let src = r".\misc";
    let rm_multiline_comment = true;
    let targets = vec!["TODO:", "FIXME:", "EDIT:", "HACK:", "INFO:", "[START]", "[END]"];
    // [END] set up params

    let src = String::from(src);  // TODO: 240217 ベースディレクトリを指定すると、その配下の .py ファイルを対象とするようにする。
    let src = remove_head_and_tail_double_quotation(&src);  // HACK: 240219 タイミングは要検討
    let src_ = Path::new(&src);

    let mut temp_dst = PathBuf::from(r".\dst_rmc");
    let now: String = Local::now()
        .format("%Y%m%d_%H%M%S")
        .to_string();
    temp_dst.push(&now);
    
    if src_.is_file() {
        temp_dst.push(src_.file_name().unwrap());
        let dst = temp_dst.to_string_lossy().to_string();
        remove_comment_and_save_one(&src, &dst, &targets, &rm_multiline_comment);

    } else if src_.is_dir() {
        let folder_name = src_.file_name().unwrap();
        temp_dst.push(folder_name);
        let dst_base_dir = temp_dst.to_string_lossy().to_string();
        
        let path_vec = retrieve_path_vec(&src);
        for fpath in path_vec {
            let dst = fpath.replace(&src, &dst_base_dir);
            remove_comment_and_save_one(&fpath, &dst, &targets, &rm_multiline_comment);
        }

    } else {
        // TODO: 240220 ファイルパスが存在しない可能性があることを、ユーザーに通知する。
        panic!("FetalError: unknown type of error ...");  // INFO: 240219 ファイルでもディレクトリでもなく、ファイルが破損している場合
    }
}

// HACK: 240217 requirements.txt を元に環境を作って、pyinstaller でビルドまでできるといいかも？
fn remove_comment_and_save_one(src: &String, dst: &String, targets: &Vec<&str>, rm_multiline_comment: &bool) {  // TODO: 240220 ソースコード以外の対応を検討する。
    let mut code = open_file(&src);
    code = rmc::py::remove_comment(&code, &targets);  // TODO: 240219 python 以外のコードにも対応すること。(拡張子で分岐する。)

    if *rm_multiline_comment {
        code = rmc::py::remove_multiline_comment(&code);
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
    result  // HACK: 240220 単体テストユニットを作るといいと思う。
}

fn open_file(path: &String) -> String {
    let mut f = File::open(path).unwrap();
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