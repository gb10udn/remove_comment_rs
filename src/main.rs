use std::io::{prelude::*, Write};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use chrono::Local;
mod rmc;


 fn main() {
    // [START] set up params
    let path = "./misc/sample_001.py";  // TODO: 240218 ここをユーザー入力 or プログラムの置かれたカレントディレクトリにする。
    let rm_multiline_comment = true;
    let targets = vec!["TODO:", "FIXME:", "EDIT:", "HACK:", "INFO:", "[START]", "[END]"];
    // [END] set up params

    let path = String::from(path);  // TODO: 240217 ベースディレクトリを指定すると、その配下の .py ファイルを対象とするようにする。
    let path = remove_head_and_tail_double_quotation(&path);

    remove_comment_and_save(&path, "", targets, rm_multiline_comment);
}

// HACK: 240217 requirements.txt を元に環境を作って、pyinstaller でビルドまでできるといいかも？
fn remove_comment_and_save(path: &String, base_dir: &str, targets: Vec<&str>, rm_multiline_comment: bool) {
    let mut code = open_file(&path);
    code = rmc::py::remove_comment(&code, targets);

    if rm_multiline_comment {
        code = rmc::py::remove_multiline_comment(&code);
    }

    let path_ = Path::new(&path);  // HACK: 240219 PathBuf でなくて、Path でも同じことできんか？

    let fname = path_
        .file_name()
        .unwrap()
        .to_string_lossy();

    let now = Local::now()
        .format("%Y%m%d_%H%M%S")
        .to_string();
    let dst = format!(r".\dst_rmc\{}\{}", now, fname);  // EDIT: 240219 base_dir が存在する場合に、パスの挿入を実行する。

    let dst_ = Path::new(&dst);
    let base_path = dst_
        .parent()
        .unwrap()
        .to_string_lossy();
    fs::create_dir_all(base_path.to_string()).unwrap();  // HACK: 240218 (あまり考えられないが) 重複したフォルダを操作する場合に処理止めていいかも？

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