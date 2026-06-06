use crate::{
    ir::{BinOp, Ident, Literal, Pat, UnOp},
    source_map::Span,
};

#[derive(Debug)]
pub(crate) enum Expr {
    Lit(Literal),
    Ident(Ident),
    Unary(ExprUnary),
    Binary(ExprBinary),
    If(ExprIf),
    Case(ExprCase),
    Let(ExprLet),
    Apply(ExprApp),
}

impl Expr {
    pub(crate) fn span(&self) -> Span {
        match self {
            Self::Lit(literal) => literal.span,
            Self::Ident(ident) => ident.span,
            Self::Unary(expr_unary) => expr_unary.span,
            Self::Binary(expr_binary) => expr_binary.span,
            Self::If(expr_if) => expr_if.span,
            Self::Case(expr_case) => expr_case.span,
            Self::Let(expr_let) => expr_let.span,
            Self::Apply(expr_app) => expr_app.span,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ExprUnary {
    pub(crate) op: UnOp,
    pub(crate) expr: Box<Expr>,
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) struct ExprBinary {
    pub(crate) lhs: Box<Expr>,
    pub(crate) op: BinOp,
    pub(crate) rhs: Box<Expr>,
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) struct ExprIf {
    pub(crate) cond: Box<Expr>,
    pub(crate) do_branch: Box<Expr>,
    pub(crate) else_branch: Option<Box<Expr>>,
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) struct ExprCase {
    pub(crate) expr: Box<Expr>,
    pub(crate) arms: Vec<(Pat, Expr)>,
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) struct ExprLet {
    pub(crate) lhs: Ident,
    pub(crate) args: Vec<Ident>,
    pub(crate) rhs: Box<Expr>,
    pub(crate) body: Box<Expr>,
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) struct ExprApp {
    pub(crate) func: Box<Expr>,
    pub(crate) arg: Box<Expr>,
    pub(crate) span: Span,
}
