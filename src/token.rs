use std::fmt::Write;

use crate::source_map::{SourceMap, Span};

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

impl Token {
    pub(crate) fn show<W: Write>(&self, mut w: W, source_map: &SourceMap) -> std::fmt::Result {
        match &self.kind {
            TokenKind::LeftParen => w.write_str("`(`")?,
            TokenKind::RightParen => w.write_str("`)`")?,
            TokenKind::Comma => w.write_str("`,`")?,
            TokenKind::Dot => w.write_str("`.`")?,
            TokenKind::Plus => w.write_str("`+`")?,
            TokenKind::Semicolon => w.write_str("`;`")?,
            TokenKind::Minus => w.write_str("`-`")?,
            TokenKind::Arrow => w.write_str("`->`")?,
            TokenKind::Star => w.write_str("`*`")?,
            TokenKind::Bang => w.write_str("`!`")?,
            TokenKind::BangEqual => w.write_str("`!=`")?,
            TokenKind::Equal => w.write_str("`=`")?,
            TokenKind::EqualEqual => w.write_str("`==`")?,
            TokenKind::Greater => w.write_str("`>`")?,
            TokenKind::GreaterEqual => w.write_str("`>=`")?,
            TokenKind::Less => w.write_str("`<`")?,
            TokenKind::LessEqual => w.write_str("`<=`")?,
            TokenKind::AndAnd => w.write_str("`&&`")?,
            TokenKind::OrOr => w.write_str("`||`")?,
            TokenKind::Slash => w.write_str("`\\`")?,
            TokenKind::Ident(ident) => write!(w, "`{ident}`")?,
            TokenKind::String(string) => write!(w, "`\"{string}\"`")?,
            TokenKind::Integer(int) => write!(w, "`{int}`")?,
            TokenKind::Float(float) => write!(w, "`{float}`")?,
            TokenKind::Else => w.write_str("`else`")?,
            TokenKind::False => w.write_str("`false`")?,
            TokenKind::If => w.write_str("`if`")?,
            TokenKind::Case => w.write_str("`case`")?,
            TokenKind::True => w.write_str("`true`")?,
            TokenKind::Return => w.write_str("`return`")?,
            TokenKind::Let => w.write_str("`let`")?,
            TokenKind::Do => w.write_str("`do`")?,
            TokenKind::End => w.write_str("`end`")?,
            TokenKind::EOF => w.write_str("`EOF`")?,
        }
        let (start_line, start_col) = source_map.map_offset(self.span.start());
        let (end_line, end_col) = source_map.map_offset(self.span.end());
        write!(
            w,
            " from {}:{} to {}:{}",
            start_line + 1,
            start_col + 1,
            end_line + 1,
            end_col + 1
        )
    }
}
