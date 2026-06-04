mod expr;

pub(crate) use crate::ast::{BinOp, Literal, UnOp};
use crate::def_gen;
pub(crate) use crate::ir::expr::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct DefId(usize);

impl DefId {
    pub(crate) const RIDICULOUS: Self = Self(usize::MAX);
}

def_gen!(DefIdGen => DefId);

#[derive(Debug)]
pub(crate) struct LetBinding<T> {
    pub(crate) lhs: DefId,
    pub(crate) ret_ty: T,
    pub(crate) args: Vec<(DefId, T)>,
    pub(crate) rhs: Box<Expr<T>>,
}

#[derive(Debug)]
pub(crate) enum Pat {
    Lit(Literal),
    Ident(DefId),
}
