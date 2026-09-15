use pcf_token::Token;

#[derive(Debug)]
pub struct Cursor<'a> {
    tokens: &'a [Token],
    position: usize,
}
