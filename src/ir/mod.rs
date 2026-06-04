mod expr;

pub(crate) use crate::ast::{BinOp, Literal, UnOp};
use crate::def_gen;
pub(crate) use crate::ir::expr::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct DefId(usize);

impl DefId {
    pub(crate) const RIDICULOUS: Self = Self(usize::MAX);
}

def_gen!(DefIdGen => DefId);

#[derive(Debug)]
pub(crate) enum Pat {
    Lit(Literal),
    Ident(DefId),
}
