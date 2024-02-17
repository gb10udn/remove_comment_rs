use std::io::prelude::*;
use std::fs::File;
use regex::Regex;

fn main() {

    let path = "./misc/sample_001.py";
    let rm_docstring = true;

    let path = String::from(path);  // TODO: 240217 ベースディレクトリを指定すると、その配下の .py ファイルを対象とするようにする。
    let path = remove_head_and_tail_double_quotation(&path);


    let mut code = open_file(&path);
    code = remove_comment_py(&code, vec!["TODO:", "FIXME:", "EDIT:", "HACK:", "[START]", "[END]"]);

    if rm_docstring {
        code = remove_docstring_py(&code);
    }
    println!("{}", code);  // TODO: 240217 ファイルに書き込む。
}

fn open_file(path: &String) -> String {
    let mut f = File::open(path).unwrap();
    let mut result = String::new();
    f.read_to_string(&mut result).unwrap();
    result
}

fn remove_comment_py(src: &String, targets: Vec<&str>) -> String {
    let pattern = targets
        .iter()
        .map(|keyword| format!("#\\s*{}.*", keyword))
        .collect::<Vec<String>>()
        .join("|");
    let re = Regex::new(&pattern).unwrap();
    re.replace_all(src, "").to_string()
}

fn remove_docstring_py(src: &String) -> String {
    let re = Regex::new(r#"\s*?"{3}[\s\S]*?"{3}|\s*?'{3}[\s\S]*?'{3}"#).unwrap();  // INFO: 240218 \s は空白や改行コード。\S はそれ以外。*? は非貪欲マッチ。
    let result = re.replace_all(src, "");
    result.to_string()
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_remove_head_and_tail_double_quotation() {
        use crate::remove_head_and_tail_double_quotation;

        assert_eq!(remove_head_and_tail_double_quotation(&String::from("abc\n")), String::from("abc"));
        assert_eq!(remove_head_and_tail_double_quotation(&String::from("\"abc\"\n")), String::from("abc"));
    }

    #[test]
    fn test_remove_comment_py() {
        use crate::remove_comment_py;

        let src = r#"
            import os
            print('始めるよ！')  # TODO: Fix this issue
            os.path.basename("hoge")  # FIXME: Address this problem
        "#;

        let dst = r#"
            import os
            print('始めるよ！')  
            os.path.basename("hoge")  
        "#;

        assert_eq!(remove_comment_py(&src.to_string(), vec!["TODO:", "FIXME:"]), dst.to_string());
    }

    #[test]
    fn test_remove_docstring_py() {
        use crate::remove_docstring_py;

        let src = r#"
            import datetime
            def hogeee():
                """
                this is docstring !!!
                """
                return datetime.datetime.now()
            def piyo():
                '''
                the second docstring !!!
                '''
                return 123
        "#;

        let dst = r#"
            import datetime
            def hogeee():
                return datetime.datetime.now()
            def piyo():
                return 123
        "#;

        assert_eq!(remove_docstring_py(&src.to_string()), dst.to_string());
    }
}