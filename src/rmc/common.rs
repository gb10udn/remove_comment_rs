use regex::Regex;

/// コメントを削除する regrex の pattern を返す関数。
/// 例えば、Python のコメントを削除したい場合は、comment_marker -> "#" を指定する。
pub fn remove_comment(src: &String, targets: &Vec<String>, comment_marker: &str) -> String {
    let pattern = targets
        .iter()
        .map(|keyword| format!(r"\s*?{}\s*?{}.*", comment_marker, keyword).replace("[", r"\[").replace("]", r"\]"))  // INFO: 240413 square brackets ([]) needs to escape (\).
        .collect::<Vec<String>>()
        .join("|");
    let re = Regex::new(&pattern).unwrap();
    re.replace_all(src, "").to_string()
}