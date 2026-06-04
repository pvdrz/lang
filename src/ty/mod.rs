use std::fmt;

use crate::def_gen;

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Ty {
    Var(VarTy),
    Int,
    Float,
    String,
    Bool,
    Unit,
    Never,
    Fn(FnTy),
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct FnTy {
    pub(crate) arg: Box<Ty>,
    pub(crate) ret: Box<Ty>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct VarTy(usize);

def_gen!(VarTyGen => VarTy);

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(var) => var.fmt(f),
            Self::Int => f.write_str("Int"),
            Self::Float => f.write_str("Float"),
            Self::String => f.write_str("String"),
            Self::Bool => f.write_str("Bool"),
            Self::Unit => f.write_str("Unit"),
            Self::Never => f.write_str("Never"),
            Self::Fn(fn_ty) => fn_ty.fmt(f),
        }
    }
}

impl fmt::Display for VarTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "X{}", self.0)
    }
}

impl fmt::Display for FnTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} -> {})", self.arg, self.ret)
    }
}
