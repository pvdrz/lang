mod expr;
mod visitor;

use crate::Lang;
pub(crate) use crate::ast::{BinOp, Literal, LiteralKind, UnOp};
pub(crate) use crate::ir::expr::*;
use crate::{def_gen, source_map::Span};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct DefId(usize);

impl DefId {
    pub(crate) fn display<'ctx>(&self, lang: &'ctx Lang) -> DisplayDefId<'ctx> {
        DisplayDefId {
            def_id: *self,
            lang,
        }
    }
}

impl DefId {
    pub(crate) const RIDICULOUS: Self = Self(usize::MAX);
}

def_gen!(DefIdGen => DefId);

pub(crate) struct DisplayDefId<'ctx> {
    def_id: DefId,
    lang: &'ctx Lang,
}

impl std::fmt::Display for DisplayDefId<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&*self.lang.def_id_name(self.def_id))
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Ident {
    pub(crate) def_id: DefId,
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) enum Pat {
    Lit(Literal),
    Ident(Ident),
}
