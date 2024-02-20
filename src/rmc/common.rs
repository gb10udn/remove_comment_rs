use regex::Regex;

/// コメントを削除する regrex の pattern を返す関数。
/// 例えば、Python のコメントを削除したい場合は、comment_marker -> "#" を指定する。
pub fn remove_comment(src: &String, targets: &Vec<&str>, comment_marker: &str) -> String {  // TODO: 240218 ps / vba にも対応すること。(vba を引き抜いて変換して、また、.xlsm に戻すのもいいかも？)
    let pattern = targets
        .iter()
        .map(|keyword| format!(r"\s*?{}\s*?{}.*", comment_marker, keyword))
        .collect::<Vec<String>>()
        .join("|");
    let re = Regex::new(&pattern).unwrap();
    re.replace_all(src, "").to_string()  // TODO: 240218 削除後に空白のみになった場合、その行を削除する？
}