
pub fn parse_result_type(s: &str) -> Option<String> {
    let s = s.trim();
    let s = s.replace(" ", "");
    if s.starts_with("Result<") && s.ends_with(",Error>") {
        let inner = &s[7..s.len() - 7];
        Some(inner.to_string())
    } else {
        None
    }
}