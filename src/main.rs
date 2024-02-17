use std::io::prelude::*;
use std::fs::File;
use regex::Regex;

fn main() {
    let path = String::from("./misc/sample_001.py");
    let path = remove_head_and_tail_double_quotation(&path);
    let code = open_file(&path);
    let code_ = remove_comment_py(&code, vec!["TODO:", "FIXME:"]);

    println!("{}", code_);
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
    let regex = Regex::new(&pattern).unwrap();
    println!("{:?}", pattern);
    regex.replace_all(src, "").to_string()
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
}