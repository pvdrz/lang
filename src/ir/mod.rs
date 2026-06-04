mod expr;

pub(crate) use crate::ast::{BinOp, Literal, UnOp};
pub(crate) use crate::ir::expr::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct DefId(usize);

impl DefId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
pub(crate) struct LetBinding<T> {
    pub(crate) lhs: DefId,
    pub(crate) ret_ty: T,
    pub(crate) args: Vec<(DefId, T)>,
    pub(crate) rhs: Box<Expr<T>>,
}

#[derive(Debug)]
pub(crate) struct File<T> {
    pub(crate) bindings: Vec<LetBinding<T>>,
}

#[derive(Debug)]
pub(crate) enum Pat {
    Lit(Literal),
    Ident(DefId),
}
