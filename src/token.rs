use crate::source_map::Span;

#[derive(Debug, PartialEq)]
pub(crate) enum TokenKind {
    LeftParen,
    RightParen,
    Comma,
    Dot,
    Plus,
    Semicolon,
    Minus,
    Arrow,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    AndAnd,
    OrOr,
    Slash,
    Ident(String),
    String(String),
    Integer(isize),
    Float(f64),
    Else,
    False,
    If,
    Case,
    True,
    Return,
    Let,
    Do,
    End,
    EOF,
}

#[derive(Debug)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) span: Span,
}
