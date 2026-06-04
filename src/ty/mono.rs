use std::fmt;

use crate::def_gen;

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum MonoTy {
    Var(VarMonoTy),
    Int,
    Float,
    String,
    Bool,
    Unit,
    Never,
    Fn(FnMonoTy),
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct FnMonoTy {
    pub(crate) arg: Box<MonoTy>,
    pub(crate) ret: Box<MonoTy>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct VarMonoTy(usize);

def_gen!(VarMonoTyGen => VarMonoTy);

impl fmt::Display for MonoTy {
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

impl fmt::Display for VarMonoTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "X{}", self.0)
    }
}

impl fmt::Display for FnMonoTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} -> {})", self.arg, self.ret)
    }
}
