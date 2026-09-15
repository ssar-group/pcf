use crate::TokenKind;

pub fn keyword_kind(ident: &str) -> Option<TokenKind> {
    Some(match ident {
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "null" => TokenKind::Null,
        "output" => TokenKind::Output,
        "import" => TokenKind::Import,
        "export" => TokenKind::Export,
        "module" => TokenKind::Module,
        "let" => TokenKind::Let,
        "const" => TokenKind::Const,
        "fn" => TokenKind::Fn,
        "return" => TokenKind::Return,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "for" => TokenKind::For,
        "while" => TokenKind::While,
        "in" => TokenKind::In,
        _ => return None,
    })
}
