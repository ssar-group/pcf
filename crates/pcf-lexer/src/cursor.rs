#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    source: &'a str,
    offset: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, offset: 0 }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn is_eof(&self) -> bool {
        self.offset >= self.source.len()
    }

    pub fn peek(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    pub fn peek_next(&self) -> Option<char> {
        self.source[self.offset..].chars().nth(1)
    }

    pub fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    pub fn consume_while(&mut self, mut predicate: impl FnMut(char) -> bool) -> &'a str {
        let start = self.offset;
        while let Some(ch) = self.peek() {
            if !predicate(ch) {
                break;
            }
            let _ = self.advance();
        }
        &self.source[start..self.offset]
    }
}
