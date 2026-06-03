use crate::ast::{
    Ident, LetBinding, Literal, Pat,
    op::{BinOp, UnOp},
};

#[derive(Debug)]
pub(crate) enum Expr {
    Lit(Literal),
    Ident(Ident),
    Unary(ExprUnary),
    Binary(ExprBinary),
    Group(ExprGroup),
    If(ExprIf),
    Case(ExprCase),
    Let(ExprLet),
    Apply(ExprApp),
}

#[derive(Debug)]
pub(crate) struct ExprUnary {
    pub(crate) op: UnOp,
    pub(crate) expr: Box<Expr>,
}

#[derive(Debug)]
pub(crate) struct ExprBinary {
    pub(crate) lhs: Box<Expr>,
    pub(crate) op: BinOp,
    pub(crate) rhs: Box<Expr>,
}

#[derive(Debug)]
pub(crate) struct ExprGroup {
    pub(crate) expr: Box<Expr>,
}

#[derive(Debug)]
pub(crate) struct ExprIf {
    pub(crate) cond: Box<Expr>,
    // This could be a block of statements
    pub(crate) then_branch: Box<Expr>,
    // This could be a block of statements
    pub(crate) else_branch: Option<Box<Expr>>,
}

#[derive(Debug)]
pub(crate) struct ExprCase {
    pub(crate) expr: Box<Expr>,
    pub(crate) arms: Vec<(Pat, Expr)>,
}

#[derive(Debug)]
pub(crate) struct ExprLet {
    pub(crate) binding: LetBinding,
    pub(crate) tail: Box<Expr>,
}

#[derive(Debug)]
pub(crate) struct ExprApp {
    pub(crate) func: Box<Expr>,
    pub(crate) arg: Box<Expr>,
}
