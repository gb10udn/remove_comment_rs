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

    let mut dst = PathBuf::from(r".\dst_rmc");
    let now: String = Local::now()
        .format("%Y%m%d_%H%M%S")
        .to_string();
    dst.push(&now);
    
    if src_.is_file() {
        dst.push(src_.file_name().unwrap());
        let dst = dst.to_string_lossy().to_string();
        remove_comment_and_save_one(&src, &dst, &targets, &rm_multiline_comment);

    } else if src_.is_dir() {
        let folder_name = src_.file_name().unwrap().to_string_lossy().to_string();
        dst.push(folder_name);
        let dst_base_dir = dst.to_string_lossy().to_string();
        for entry in WalkDir::new(&src) {  // HACK: 240219 ネストが深くなっているので、ファイル一覧を取得するのがいいかも？
            if let Ok(val) = entry {
                if val.path().is_file() {  // FIXME: 240219 これで判定できないケースがなかったか確認せよ。(フォルダ名で、hoge.txt で誤判定しなかったっけ？)
                    let fpath = val.path().to_string_lossy().to_string();
                    let dst = fpath.replace(&src, &dst_base_dir);
                    println!("fpath = {}", fpath);
                    remove_comment_and_save_one(&fpath, &dst, &targets, &rm_multiline_comment);
                }
            }
        }
    } else {
        panic!("FetalError: unknown type of error ...");  // INFO: 240219 ファイルでもディレクトリでもなく、ファイルが破損している場合。
    }
}

// HACK: 240217 requirements.txt を元に環境を作って、pyinstaller でビルドまでできるといいかも？
fn remove_comment_and_save_one(src: &String, dst: &String, targets: &Vec<&str>, rm_multiline_comment: &bool) {
    let mut code = open_file(&src);
    code = rmc::py::remove_comment(&code, &targets);  // TODO: 240219 python 以外のコードにも対応すること。

    if *rm_multiline_comment {
        code = rmc::py::remove_multiline_comment(&code);
    }

    // [START] create dist basedir
    let dst_ = Path::new(dst);
    let base_path = dst_
        .parent()
        .unwrap()
        .to_string_lossy();
    fs::create_dir_all(base_path.to_string()).unwrap();  // HACK: 240218 (あまり考えられないが) 重複したフォルダを操作する場合に処理止めていいかも？
    // [END] create dist basedir
    
    let mut file = File::create(dst)
        .expect("file not found.");  

    write!(file, "{}", code)
        .expect("cannot write.");
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