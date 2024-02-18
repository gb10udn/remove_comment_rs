use std::io::{prelude::*, Write};
use std::fs::{self, File};
use std::path::PathBuf;
use chrono::Local;
mod rmc;


 fn main() {
    let path = "./misc/sample_001.py";  // TODO: 240218 ここをユーザー入力 or プログラムの置かれたカレントディレクトリにする。
    let rm_docstring = true;

    let path = String::from(path);  // TODO: 240217 ベースディレクトリを指定すると、その配下の .py ファイルを対象とするようにする。
    let path = remove_head_and_tail_double_quotation(&path);

    let mut code = open_file(&path);
    code = rmc::py::remove_comment(&code, vec!["TODO:", "FIXME:", "EDIT:", "HACK:", "INFO:", "[START]", "[END]"]);

    if rm_docstring {
        code = rmc::py::remove_multiline_comment(&code);
    }
    println!("{}", code);  // HACK: 240217 requirements.txt を元に環境を作って、pyinstaller でビルドまでできるといいかも？


    // TODO: 240218 複数ディレクトリの場合に、ベースディレクトリからのディレクトリ構成にすること。
    let path_buf = PathBuf::from(&path);

    let fname = path_buf
        .file_name()
        .unwrap()
        .to_string_lossy();

    let now = Local::now()
        .format("%Y%m%d_%H%M%S")
        .to_string();
    let dst = format!(r".\dst_rmc\{}\{}", now, fname);

    println!("{}", dst);  // -> うまく取得できていそう。

    
    let path_buf = PathBuf::from(&dst);
    let base_path = path_buf
        .parent()
        .unwrap()
        .to_string_lossy();
    println!("base_path = {}", base_path);  // -> うまく取得できていそう。
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