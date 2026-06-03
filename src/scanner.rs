use crate::{
    Lang,
    token::{Token, TokenKind},
};

pub struct Scanner<'ctx> {
    source: Vec<char>,
    tokens: Vec<Token>,
    // First character in the lexeme being scanned.
    start: usize,
    // The character that is currently being considered.
    current: usize,
    line: usize,
    lang: &'ctx Lang,
}

impl<'ctx> Scanner<'ctx> {
    pub(crate) fn new(source: String, lang: &'ctx Lang) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            lang,
        }
    }

    pub(crate) fn scan_tokens(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()
        }

        self.tokens.push(Token {
            kind: TokenKind::EOF,
            line: self.line,
        });
        self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            ',' => self.add_token(TokenKind::Comma),
            '.' => self.add_token(TokenKind::Dot),
            '+' => self.add_token(TokenKind::Plus),
            ';' => self.add_token(TokenKind::Semicolon),
            '*' => self.add_token(TokenKind::Star),
            '-' => {
                let kind = if self.matches('>') {
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                };
                self.add_token(kind)
            }
            '!' => {
                let kind = if self.matches('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                };
                self.add_token(kind)
            }
            '=' => {
                let kind = if self.matches('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                };
                self.add_token(kind)
            }
            '<' => {
                let kind = if self.matches('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                };
                self.add_token(kind)
            }
            '>' => {
                let kind = if self.matches('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                };
                self.add_token(kind)
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenKind::Slash);
                }
            }
            '&' => {
                if self.matches('&') {
                    self.add_token(TokenKind::AndAnd)
                } else {
                    self.lang.error(self.line, format!("Expected `&`"))
                }
            }
            '|' => {
                if self.matches('|') {
                    self.add_token(TokenKind::OrOr)
                } else {
                    self.lang.error(self.line, format!("Expected `|`"))
                }
            }

            '"' => self.string(),
            c if c.is_digit(10) => self.number(),
            ' ' | '\r' | '\t' => {}
            c if c.is_alphabetic() || c == '_' => self.identifier(),
            '\n' => {
                self.line += 1;
            }
            c => self
                .lang
                .error(self.line, format!("Unexpected character `{c}`.")),
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        c
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            line: self.line,
        })
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;

        true
    }

    fn peek(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source[self.current]
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.lang.error(self.line, "Unterminated string.");
            return;
        }
        // This is a `"` because we are not at the end of the source and the while loop above
        // stopped.
        self.advance();

        self.add_token(TokenKind::String(
            self.get_substring(self.start + 1, self.current - 1),
        ));
    }

    fn get_substring(&self, lo: usize, hi: usize) -> String {
        self.source[lo..hi].iter().copied().collect()
    }

    fn number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }
        let mut is_float = false;

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            is_float = true;
            self.advance();
        }

        while self.peek().is_digit(10) {
            self.advance();
        }

        let substring = self.get_substring(self.start, self.current);
        if is_float {
            self.add_token(TokenKind::Float(substring.parse().unwrap()));
        } else {
            self.add_token(TokenKind::Integer(substring.parse().unwrap()));
        }
    }

    fn peek_next(&mut self) -> char {
        if self.current + 1 > self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn identifier(&mut self) {
        while {
            let c = self.peek();
            c.is_alphanumeric() || c == '_'
        } {
            self.advance();
        }

        let ident = self.get_substring(self.start, self.current);
        let kind = match ident.as_str() {
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "if" => TokenKind::If,
            "case" => TokenKind::Case,
            "true" => TokenKind::True,
            "return" => TokenKind::Return,
            "let" => TokenKind::Let,
            "do" => TokenKind::Do,
            "end" => TokenKind::End,
            _ => TokenKind::Ident(ident),
        };
        self.add_token(kind);
    }
}
