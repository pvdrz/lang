use std::fmt::Display;

use crate::{
    Lang,
    ast::{
        BinOp, Expr, ExprApp, ExprBinary, ExprIf, ExprLet, ExprUnary, Ident, LetBinding, Literal,
        Pat, UnOp,
    },
    token::{Token, TokenKind},
};

pub(crate) struct Parser<'ctx> {
    tokens: Vec<Token>,
    current: usize,
    lang: &'ctx Lang,
}

impl<'ctx> Parser<'ctx> {
    pub(crate) fn new(tokens: Vec<Token>, lang: &'ctx Lang) -> Self {
        Self {
            tokens,
            current: 0,
            lang,
        }
    }

    pub(crate) fn parse(&mut self) -> Result<Expr, ParseError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        if self.matches(&TokenKind::If) {
            self.conditional()
        } else if self.matches(&TokenKind::Case) {
            self.case()
        } else if self.matches(&TokenKind::Let) {
            let binding = self.let_binding()?;
            let tail = self.expression()?;
            Ok(Expr::Let(ExprLet {
                binding,
                tail: Box::new(tail),
            }))
        } else {
            self.logic_or()
        }
    }

    fn conditional(&mut self) -> Result<Expr, ParseError> {
        let cond = self.expression()?;

        self.consume(&TokenKind::Do, "Expected `do` in conditional.")?;
        let then_branch = self.expression()?;

        let mut else_branch = None;

        if self.matches(&TokenKind::Else) {
            else_branch = Some(Box::new(self.expression()?));
        }

        self.consume(&TokenKind::End, "Expected `end` in conditional.")?;

        Ok(Expr::If(ExprIf {
            cond: Box::new(cond),
            do_branch: Box::new(then_branch),
            else_branch,
        }))
    }

    fn case(&mut self) -> Result<Expr, ParseError> {
        let expr = self.expression()?;

        self.consume(&TokenKind::Do, "Expected `do` in case.")?;

        let mut arms = Vec::new();

        while !self.matches(&TokenKind::End) {
            let pat = self.pat()?;

            self.consume(&TokenKind::Arrow, "Expected `->` in case.")?;

            let expr = self.expression()?;

            self.consume(&TokenKind::Comma, "Expected `,` in case.")?;

            arms.push((pat, expr));
        }

        Ok(Expr::Case(crate::ast::ExprCase {
            expr: Box::new(expr),
            arms,
        }))
    }

    fn pat(&mut self) -> Result<Pat, ParseError> {
        let pat = if let Some(ident) = self.matches_pattern(match_ident) {
            Pat::Ident(ident)
        } else if self.matches(&TokenKind::False) {
            Pat::Lit(Literal::False)
        } else if self.matches(&TokenKind::True) {
            Pat::Lit(Literal::True)
        } else if let Some(int) = self.matches_pattern(|tk| match &tk.kind {
            TokenKind::Integer(int) => Some(*int),
            _ => None,
        }) {
            Pat::Lit(Literal::Int(int))
        } else if let Some(float) = self.matches_pattern(|tk| match &tk.kind {
            TokenKind::Float(float) => Some(*float),
            _ => None,
        }) {
            Pat::Lit(Literal::Float(float))
        } else if let Some(s) = self.matches_pattern(|tk| match &tk.kind {
            TokenKind::String(s) => Some(s.clone()),
            _ => None,
        }) {
            Pat::Lit(Literal::Str(s))
        } else {
            return self.err(self.peek(), "Expected pattern in case.");
        };

        Ok(pat)
    }

    fn let_binding(&mut self) -> Result<LetBinding, ParseError> {
        if let Some(lhs) = self.matches_pattern(match_ident) {
            let mut args = vec![];
            while {
                if let Some(ident) = self.matches_pattern(match_ident) {
                    args.push(ident);

                    true
                } else {
                    false
                }
            } {}

            self.consume(&TokenKind::Equal, "Expected `=` in let binding.")?;
            let rhs = self.expression()?;

            self.consume(&TokenKind::Semicolon, "Expected `;` in let binding.")?;

            Ok(LetBinding {
                lhs,
                args,
                rhs: Box::new(rhs),
            })
        } else {
            self.err(self.peek(), "Expected identifier in let binding.")
        }
    }

    fn logic_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logic_and()?;

        while {
            match self.matches_pattern(|tk| match &tk.kind {
                TokenKind::OrOr => Some(BinOp::Or),
                _ => None,
            }) {
                Some(op) => {
                    let rhs = self.logic_and()?;
                    expr = Expr::Binary(ExprBinary {
                        lhs: Box::new(expr),
                        op,
                        rhs: Box::new(rhs),
                    });

                    true
                }
                None => false,
            }
        } {}

        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while {
            match self.matches_pattern(|tk| match &tk.kind {
                TokenKind::AndAnd => Some(BinOp::And),
                _ => None,
            }) {
                Some(op) => {
                    let rhs = self.equality()?;
                    expr = Expr::Binary(ExprBinary {
                        lhs: Box::new(expr),
                        op,
                        rhs: Box::new(rhs),
                    });

                    true
                }
                None => false,
            }
        } {}

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while {
            match self.matches_pattern(|tk| match &tk.kind {
                TokenKind::BangEqual => Some(BinOp::Ne),
                TokenKind::EqualEqual => Some(BinOp::Eq),
                _ => None,
            }) {
                Some(op) => {
                    let rhs = self.comparison()?;
                    expr = Expr::Binary(ExprBinary {
                        lhs: Box::new(expr),
                        op,
                        rhs: Box::new(rhs),
                    });

                    true
                }
                None => false,
            }
        } {}

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while {
            match self.matches_pattern(|tk| match &tk.kind {
                TokenKind::Greater => Some(BinOp::Gt),
                TokenKind::GreaterEqual => Some(BinOp::Ge),
                TokenKind::Less => Some(BinOp::Lt),
                TokenKind::LessEqual => Some(BinOp::Le),
                _ => None,
            }) {
                Some(op) => {
                    let rhs = self.term()?;
                    expr = Expr::Binary(ExprBinary {
                        lhs: Box::new(expr),
                        op,
                        rhs: Box::new(rhs),
                    });

                    true
                }
                None => false,
            }
        } {}

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while {
            match self.matches_pattern(|tk| match &tk.kind {
                TokenKind::Minus => Some(BinOp::Sub),
                TokenKind::Plus => Some(BinOp::Add),
                _ => None,
            }) {
                Some(op) => {
                    let rhs = self.factor()?;
                    expr = Expr::Binary(ExprBinary {
                        lhs: Box::new(expr),
                        op,
                        rhs: Box::new(rhs),
                    });

                    true
                }
                None => false,
            }
        } {}

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        while {
            match self.matches_pattern(|tk| match &tk.kind {
                TokenKind::Slash => Some(BinOp::Div),
                TokenKind::Star => Some(BinOp::Mul),
                _ => None,
            }) {
                Some(op) => {
                    let rhs = self.unary()?;
                    expr = Expr::Binary(ExprBinary {
                        lhs: Box::new(expr),
                        op,
                        rhs: Box::new(rhs),
                    });

                    true
                }
                None => false,
            }
        } {}

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if let Some(op) = self.matches_pattern(|tk| match &tk.kind {
            TokenKind::Bang => Some(UnOp::Not),
            TokenKind::Minus => Some(UnOp::Neg),
            _ => None,
        }) {
            let expr = self.unary()?;

            Ok(Expr::Unary(ExprUnary {
                op,
                expr: Box::new(expr),
            }))
        } else {
            self.application()
        }
    }

    fn application(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        while let TokenKind::False
        | TokenKind::True
        | TokenKind::Integer(_)
        | TokenKind::Float(_)
        | TokenKind::String(_)
        | TokenKind::Ident(_)
        | TokenKind::LeftParen = &self.peek().kind
        {
            let arg = self.primary()?;
            expr = Expr::Apply(ExprApp {
                func: Box::new(expr),
                arg: Box::new(arg),
            })
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let expr = if self.matches(&TokenKind::False) {
            Expr::Lit(Literal::False)
        } else if self.matches(&TokenKind::True) {
            Expr::Lit(Literal::True)
        } else if let Some(int) = self.matches_pattern(|tk| match &tk.kind {
            TokenKind::Integer(int) => Some(*int),
            _ => None,
        }) {
            Expr::Lit(Literal::Int(int))
        } else if let Some(float) = self.matches_pattern(|tk| match &tk.kind {
            TokenKind::Float(float) => Some(*float),
            _ => None,
        }) {
            Expr::Lit(Literal::Float(float))
        } else if let Some(s) = self.matches_pattern(|tk| match &tk.kind {
            TokenKind::String(s) => Some(s.clone()),
            _ => None,
        }) {
            Expr::Lit(Literal::Str(s))
        } else if let Some(ident) = self.matches_pattern(match_ident) {
            Expr::Ident(ident)
        } else if self.matches(&TokenKind::LeftParen) {
            let expr = self.expression()?;
            self.consume(&TokenKind::RightParen, "Expected ')' after epression.")?;
            Expr::Group(crate::ast::ExprGroup {
                expr: Box::new(expr),
            })
        } else {
            return self.err(self.peek(), "Expected expression.");
        };

        Ok(expr)
    }

    fn matches_pattern<T, F: Fn(&Token) -> Option<T>>(&mut self, pat: F) -> Option<T> {
        if let Some(output) = self.check_pattern(pat) {
            self.advance();
            Some(output)
        } else {
            None
        }
    }

    fn matches(&mut self, kind: &TokenKind) -> bool {
        self.matches_pattern(|tk| (&tk.kind == kind).then_some(()))
            .is_some()
    }

    fn check_pattern<T, F: Fn(&Token) -> Option<T>>(&self, pat: F) -> Option<T> {
        if self.is_at_end() {
            None
        } else {
            pat(&self.peek())
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.check_pattern(|k| (&k.kind == kind).then_some(()))
            .is_some()
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, kind: &TokenKind, msg: impl Display) -> Result<&Token, ParseError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            self.err(self.peek(), msg)
        }
    }

    fn err<T>(&self, token: &Token, msg: impl Display) -> Result<T, ParseError> {
        self.lang.parse_error(token, msg);
        Err(ParseError {})
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }

            if let TokenKind::Let | TokenKind::If | TokenKind::Case | TokenKind::Return =
                self.peek().kind
            {
                return;
            }

            self.advance();
        }
    }
}

#[derive(Debug)]
pub(crate) struct ParseError {}

fn match_ident(tk: &Token) -> Option<Ident> {
    match &tk.kind {
        TokenKind::Ident(ident) => Some(Ident::new(ident.clone(), tk.line)),
        _ => None,
    }
}
