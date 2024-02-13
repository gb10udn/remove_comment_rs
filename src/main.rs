fn main() {
    println!("Hello, world!");
}

fn remove_head_and_tail_double_quotation(arg: String) -> String {
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

        assert_eq!(remove_head_and_tail_double_quotation(String::from("abc\n")), String::from("abc"));
        assert_eq!(remove_head_and_tail_double_quotation(String::from("\"abc\"\n")), String::from("abc"));
    }
}