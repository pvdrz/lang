use crate::ir::{BinOp, DefId, LetBinding, Literal, Pat, UnOp};

#[derive(Debug)]
pub(crate) enum Expr<T> {
    Lit(Literal),
    Ident(DefId),
    Unary(ExprUnary<T>),
    Binary(ExprBinary<T>),
    Group(ExprGroup<T>),
    If(ExprIf<T>),
    Case(ExprCase<T>),
    Let(ExprLet<T>),
    Apply(ExprApp<T>),
}

#[derive(Debug)]
pub(crate) struct ExprUnary<T> {
    pub(crate) op: UnOp,
    pub(crate) expr: Box<Expr<T>>,
}

#[derive(Debug)]
pub(crate) struct ExprBinary<T> {
    pub(crate) lhs: Box<Expr<T>>,
    pub(crate) op: BinOp,
    pub(crate) rhs: Box<Expr<T>>,
}

#[derive(Debug)]
pub(crate) struct ExprGroup<T> {
    pub(crate) expr: Box<Expr<T>>,
}

#[derive(Debug)]
pub(crate) struct ExprIf<T> {
    pub(crate) cond: Box<Expr<T>>,
    pub(crate) do_branch: Box<Expr<T>>,
    pub(crate) else_branch: Option<Box<Expr<T>>>,
}

#[derive(Debug)]
pub(crate) struct ExprCase<T> {
    pub(crate) expr: Box<Expr<T>>,
    pub(crate) arms: Vec<(Pat, Expr<T>)>,
}

#[derive(Debug)]
pub(crate) struct ExprLet<T> {
    pub(crate) binding: LetBinding<T>,
    pub(crate) tail: Box<Expr<T>>,
}

#[derive(Debug)]
pub(crate) struct ExprApp<T> {
    pub(crate) func: Box<Expr<T>>,
    pub(crate) arg: Box<Expr<T>>,
}
