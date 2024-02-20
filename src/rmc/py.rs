use regex::Regex;
use crate::rmc::common;

pub fn remove_comment(src: &String, targets: &Vec<&str>) -> String {
    let comment_marker = "#";
    common::remove_comment(src, targets, comment_marker)
}

pub fn remove_multiline_comment(src: &String) -> String {
    let re = Regex::new(r#"\s*?"{3}[\s\S]*?"{3}|\s*?'{3}[\s\S]*?'{3}"#).unwrap();  // INFO: 240218 \s は空白や改行コード。\S はそれ以外。*? は非貪欲マッチ。
    let result = re.replace_all(src, "");
    result.to_string()
}

#[cfg(test)]
mod tests {
 #[test]
    fn test_remove_comment() {
        use crate::rmc::py::remove_comment;

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

        assert_eq!(remove_comment(&src.to_string(), &vec!["TODO:", "FIXME:"]), dst.to_string());
    }

    #[test]
    fn test_remove_multiline_comment() {
        use crate::rmc::py::remove_multiline_comment;

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

        assert_eq!(remove_multiline_comment(&src.to_string()), dst.to_string());
    }
}