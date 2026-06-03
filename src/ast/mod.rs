mod expr;
mod op;
pub(crate) mod pretty_printer;
mod visitor;

pub(crate) use expr::*;
pub(crate) use op::*;

#[derive(Debug)]
pub(crate) struct Ident {
    inner: String,
}

impl Ident {
    pub(crate) fn new(inner: String) -> Self {
        Self { inner }
    }
}

#[derive(Debug)]
pub(crate) enum Literal {
    Int(isize),
    Float(f64),
    Str(String),
    True,
    False,
}

#[derive(Debug)]
pub(crate) struct LetBinding {
    pub(crate) lhs: Ident,
    pub(crate) args: Vec<Ident>,
    pub(crate) rhs: Box<Expr>,
}

#[derive(Debug)]
pub(crate) struct File {
    pub(crate) bindings: Vec<LetBinding>,
}

#[derive(Debug)]
pub(crate) enum Pat {
    Lit(Literal),
    Ident(Ident),
}
