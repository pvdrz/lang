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
pub(crate) struct LetBinding {
    pub(crate) lhs: DefId,
    pub(crate) args: Vec<DefId>,
    pub(crate) rhs: Box<Expr>,
}

#[derive(Debug)]
pub(crate) struct File {
    pub(crate) bindings: Vec<LetBinding>,
}

#[derive(Debug)]
pub(crate) enum Pat {
    Lit(Literal),
    Ident(DefId),
}
