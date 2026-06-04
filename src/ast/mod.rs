mod expr;
mod op;
pub(crate) mod pretty_printer;
pub(crate) mod visitor;

use std::{fmt, hash::Hash};

pub(crate) use expr::*;
pub(crate) use op::*;

use crate::source_map::Span;

#[derive(Debug, Clone)]
pub(crate) struct Ident {
    inner: String,
    span: Span,
}

impl Ident {
    pub(crate) fn new(inner: String, span: Span) -> Self {
        Self { inner, span }
    }

    pub(crate) fn span(&self) -> Span {
        self.span
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for Ident {}

impl Hash for Ident {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.inner)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Literal {
    Int(isize),
    Float(f64),
    Str(String),
    True,
    False,
}

#[derive(Debug)]
pub(crate) enum Pat {
    Lit(Literal),
    Ident(Ident),
}
