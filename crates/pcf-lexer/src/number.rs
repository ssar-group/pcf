pub fn parse_number(text: &str) -> Option<i64> {
    text.parse().ok()
}

pub fn parse_float(text: &str) -> Option<f64> {
    text.parse().ok()
}
