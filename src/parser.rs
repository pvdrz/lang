use std::fmt::Display;

use crate::{
    Lang,
    ast::ExprLet,
    ast::{
        BinOp, Expr, ExprApp, ExprBinary, ExprIf, ExprUnary, Ident, Literal, LiteralKind, Pat, UnOp,
    },
    source_map::Span,
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
        let expr = self.expression()?;
        let next_token = self.peek();
        if next_token.kind != TokenKind::EOF {
            return self.err(next_token, "Unexpected token.");
        }

        Ok(expr)
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        if let Some(span) = self.matches(&TokenKind::If) {
            self.conditional(span)
        } else if let Some(span) = self.matches(&TokenKind::Case) {
            self.case(span)
        } else if let Some(span) = self.matches(&TokenKind::Let) {
            self.let_binding(span)
        } else {
            self.logic_or()
        }
    }

    fn conditional(&mut self, lspan: Span) -> Result<Expr, ParseError> {
        let cond = self.expression()?;

        self.consume(&TokenKind::Do, "Expected `do` in conditional.")?;
        let then_branch = self.expression()?;

        let mut else_branch = None;

        if self.matches(&TokenKind::Else).is_some() {
            else_branch = Some(Box::new(self.expression()?));
        }

        let rspan = self
            .consume(&TokenKind::End, "Expected `end` in conditional.")?
            .span;

        Ok(Expr::If(ExprIf {
            cond: Box::new(cond),
            do_branch: Box::new(then_branch),
            else_branch,
            span: lspan.merge(&rspan),
        }))
    }

    fn case(&mut self, lspan: Span) -> Result<Expr, ParseError> {
        let expr = self.expression()?;

        self.consume(&TokenKind::Do, "Expected `do` in case.")?;

        let mut arms = Vec::new();

        while {
            if let Some(rspan) = self.matches(&TokenKind::End) {
                return Ok(Expr::Case(crate::ast::ExprCase {
                    span: lspan.merge(&rspan),
                    expr: Box::new(expr),
                    arms,
                }));
            } else {
                let pat = self.pat()?;

                self.consume(&TokenKind::Arrow, "Expected `->` in case.")?;

                let expr = self.expression()?;

                self.consume(&TokenKind::Comma, "Expected `,` in case.")?;

                arms.push((pat, expr));

                true
            }
        } {}

        self.err(self.peek(), "Expected `end` in case.")
    }

    fn let_binding(&mut self, lspan: Span) -> Result<Expr, ParseError> {
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

            let rspan = self
                .consume(&TokenKind::Semicolon, "Expected `;` in let binding.")?
                .span;

            let body = self.expression()?;
            Ok(Expr::Let(ExprLet {
                span: lspan.merge(&rspan),
                lhs,
                args,
                rhs: Box::new(rhs),
                body: Box::new(body),
            }))
        } else {
            self.err(self.peek(), "Expected identifier in let binding.")
        }
    }

    fn maybe_literal(&mut self) -> Option<Literal> {
        if let Some(literal) = self.matches_pattern(|tk| match &tk.kind {
            TokenKind::False => Some(Literal {
                kind: LiteralKind::False,
                span: tk.span,
            }),
            _ => None,
        }) {
            Some(literal)
        } else if let Some(literal) = self.matches_pattern(|tk| match &tk.kind {
            TokenKind::True => Some(Literal {
                kind: LiteralKind::True,
                span: tk.span,
            }),
            _ => None,
        }) {
            Some(literal)
        } else if let Some(literal) = self.matches_pattern(|tk| match &tk.kind {
            TokenKind::Integer(int) => Some(Literal {
                kind: LiteralKind::Int(*int),
                span: tk.span,
            }),
            _ => None,
        }) {
            Some(literal)
        } else if let Some(literal) = self.matches_pattern(|tk| match &tk.kind {
            TokenKind::Float(float) => Some(Literal {
                kind: LiteralKind::Float(*float),
                span: tk.span,
            }),
            _ => None,
        }) {
            Some(literal)
        } else if let Some(literal) = self.matches_pattern(|tk| match &tk.kind {
            TokenKind::String(s) => Some(Literal {
                kind: LiteralKind::Str(s.clone()),
                span: tk.span,
            }),
            _ => None,
        }) {
            Some(literal)
        } else {
            None
        }
    }

    fn pat(&mut self) -> Result<Pat, ParseError> {
        let pat = if let Some(ident) = self.matches_pattern(match_ident) {
            Pat::Ident(ident)
        } else if let Some(literal) = self.maybe_literal() {
            Pat::Lit(literal)
        } else {
            return self.err(self.peek(), "Expected pattern in case.");
        };

        Ok(pat)
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
                        span: expr.span().merge(&rhs.span()),
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
                        span: expr.span().merge(&rhs.span()),
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
                        span: expr.span().merge(&rhs.span()),
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
                        span: expr.span().merge(&rhs.span()),
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
                        span: expr.span().merge(&rhs.span()),
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
                        span: expr.span().merge(&rhs.span()),
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
        if let Some((op, lspan)) = self.matches_pattern(|tk| match &tk.kind {
            TokenKind::Bang => Some((UnOp::Not, tk.span)),
            TokenKind::Minus => Some((UnOp::Neg, tk.span)),
            _ => None,
        }) {
            let expr = self.unary()?;

            Ok(Expr::Unary(ExprUnary {
                span: lspan.merge(&expr.span()),
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
                span: expr.span().merge(&arg.span()),
                func: Box::new(expr),
                arg: Box::new(arg),
            })
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let expr = if let Some(literal) = self.maybe_literal() {
            Expr::Lit(literal)
        } else if let Some(ident) = self.matches_pattern(match_ident) {
            Expr::Ident(ident)
        } else if let Some(lspan) = self.matches_pattern(|tk| match &tk.kind {
            TokenKind::LeftParen => Some(tk.span),
            _ => None,
        }) {
            let expr = self.expression()?;

            let rspan = self
                .consume(&TokenKind::RightParen, "Expected ')' after epression.")?
                .span;

            Expr::Group(crate::ast::ExprGroup {
                expr: Box::new(expr),
                span: lspan.merge(&rspan),
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

    fn matches(&mut self, kind: &TokenKind) -> Option<Span> {
        self.matches_pattern(|tk| (&tk.kind == kind).then(|| tk.span))
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
        self.lang.error(token.span, msg);
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
        TokenKind::Ident(ident) => Some(Ident::new(ident.clone(), tk.span)),
        _ => None,
    }
}
