use crate::{
    ir::{BinOp, DefId, Literal, Pat, UnOp},
    ty::Ty,
};

#[derive(Debug)]
pub(crate) enum Expr {
    Lit(Literal),
    Ident(DefId),
    Unary(ExprUnary),
    Binary(ExprBinary),
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
pub(crate) struct ExprIf {
    pub(crate) cond: Box<Expr>,
    pub(crate) do_branch: Box<Expr>,
    pub(crate) else_branch: Option<Box<Expr>>,
}

#[derive(Debug)]
pub(crate) struct ExprCase {
    pub(crate) expr: Box<Expr>,
    pub(crate) arms: Vec<(Pat, Expr)>,
}

#[derive(Debug)]
pub(crate) struct ExprLet {
    pub(crate) lhs: DefId,
    pub(crate) ret_ty: Ty,
    pub(crate) args: Vec<(DefId, Ty)>,
    pub(crate) rhs: Box<Expr>,
    pub(crate) body: Box<Expr>,
}

#[derive(Debug)]
pub(crate) struct ExprApp {
    pub(crate) func: Box<Expr>,
    pub(crate) arg: Box<Expr>,
}
