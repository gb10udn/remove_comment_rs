use std::io::prelude::*;
use std::fs::File;
mod rmc;


 fn main() {
    let path = "./misc/sample_001.py";
    let rm_docstring = true;

    let path = String::from(path);  // TODO: 240217 ベースディレクトリを指定すると、その配下の .py ファイルを対象とするようにする。
    let path = remove_head_and_tail_double_quotation(&path);

    let mut code = open_file(&path);
    code = rmc::py::remove_comment(&code, vec!["TODO:", "FIXME:", "EDIT:", "HACK:", "INFO:", "[START]", "[END]"]);

    if rm_docstring {
        code = rmc::py::remove_docstring(&code);
    }
    println!("{}", code);  // TODO: 240217 ファイルに書き込む。 (requirements.txt を元に環境を作って、pyinstaller でビルドまでできると、いいかも？)
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