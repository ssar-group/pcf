use pcf_token::{Token, TokenKind};

#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    pub fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    pub fn current_kind(&self) -> Option<TokenKind> {
        self.current().map(|token| token.kind.clone())
    }

    pub fn peek(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.position.saturating_add(offset))
    }

    pub fn is_end(&self) -> bool {
        match self.current() {
            Some(token) => matches!(token.kind, TokenKind::Eof),
            None => true,
        }
    }

    pub fn advance(&mut self) -> Option<Token> {
        let current = self.current().cloned();
        if self.current().is_some() {
            self.position = self.position.saturating_add(1).min(self.tokens.len());
        }
        current
    }

    pub fn check(&self, kind: &TokenKind) -> bool {
        self.current().is_some_and(|token| &token.kind == kind)
    }

    pub fn matches(&self, kinds: &[TokenKind]) -> bool {
        self.current()
            .is_some_and(|token| kinds.iter().any(|kind| kind == &token.kind))
    }

    pub fn consume_if(&mut self, kind: &TokenKind) -> Option<Token> {
        if self.check(kind) {
            self.advance()
        } else {
            None
        }
    }
}
