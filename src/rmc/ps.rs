use regex::Regex;
use crate::rmc::common;

pub fn remove_comment(src: &String, targets: &Vec<&str>) -> String {
    let comment_marker = "#";
    common::remove_comment(src, targets, comment_marker)
}

pub fn remove_multiline_comment(src: &String) -> String {
    let re = Regex::new(r#"\s*?<#[\s\S]*?#>"#).unwrap();  // INFO: 240218 \s は空白や改行コード。\S はそれ以外。*? は非貪欲マッチ。
    let result = re.replace_all(src, "");
    result.to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_remove_comment() {
        use crate::rmc::ps::remove_comment;

        let src = r#"
            ping  # INFO: これは消せるかな？
            Write-Output 'ピヨピヨだよ～'  # FIXME: もっと硬派に泣くこと
        "#;

        let dst = r#"
            ping
            Write-Output 'ピヨピヨだよ～'
        "#;

        assert_eq!(remove_comment(&src.to_string(), &vec!["INFO:", "FIXME:"]), dst.to_string());
    }

    #[test]
    fn test_remove_multiline_comment() {
        use crate::rmc::ps::remove_multiline_comment;

        let src = r#"
            Function piyopiyo() {
                <#
                突っ張り気味になくぴよりん
                #>
                Write-Output 'ピヨピヨしてんじゃねーよ'
            }
        "#;

        let dst = r#"
            Function piyopiyo() {
                Write-Output 'ピヨピヨしてんじゃねーよ'
            }
        "#;

        assert_eq!(remove_multiline_comment(&src.to_string()), dst.to_string());
    }
}