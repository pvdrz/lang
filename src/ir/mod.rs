mod expr;

pub(crate) use crate::ast::{BinOp, Literal, LiteralKind, UnOp};
pub(crate) use crate::ir::expr::*;
use crate::{def_gen, source_map::Span};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct DefId(usize);

impl DefId {
    pub(crate) const RIDICULOUS: Self = Self(usize::MAX);
}

def_gen!(DefIdGen => DefId);

#[derive(Debug)]
pub(crate) struct Ident {
    pub(crate) def_id: DefId,
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) enum Pat {
    Lit(Literal),
    Ident(Ident),
}
