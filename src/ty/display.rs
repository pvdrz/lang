use super::*;
use std::fmt::{self, Debug};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Name(usize);

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "X{}", self.0)
    }
}

def_gen!(NameGen => Name);

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, &mut Vec::new(), &mut NameGen::new())
    }
}

impl Ty {
    fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        vars: &mut Vec<Vec<Name>>,
        name_gen: &mut NameGen,
    ) -> fmt::Result {
        match self {
            Self::ForAll(forall_ty) => forall_ty.display(f, vars, name_gen),
            Self::Skolem(skolem_ty) => skolem_ty.display(f, vars, name_gen),
            Self::Fn(fn_ty) => fn_ty.display(f, vars, name_gen),
            Self::Var(var) => fmt::Display::fmt(var, f),
            Self::Int => f.write_str("Int"),
            Self::Float => f.write_str("Float"),
            Self::String => f.write_str("String"),
            Self::Bool => f.write_str("Bool"),
            Self::Unit => f.write_str("Unit"),
            Self::Never => f.write_str("Never"),
        }
    }
}

impl ForAllTy {
    fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        vars: &mut Vec<Vec<Name>>,
        name_gen: &mut NameGen,
    ) -> fmt::Result {
        f.write_str("forall")?;

        let mut args = Vec::new();

        for _ in 0..self.args {
            let arg = name_gen.generate();
            write!(f, " {arg}")?;
            args.push(arg);
        }

        f.write_str(". ")?;

        vars.push(args);
        self.ty.display(f, vars, name_gen)?;
        vars.pop();

        Ok(())
    }
}

impl SkolemTy {
    fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        vars: &mut Vec<Vec<Name>>,
        _name_gen: &mut NameGen,
    ) -> fmt::Result {
        let index = vars.len() - self.debruijn - 1;
        let name = vars[index][self.index];
        name.fmt(f)
    }
}

impl FnTy {
    fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        vars: &mut Vec<Vec<Name>>,
        name_gen: &mut NameGen,
    ) -> fmt::Result {
        self.arg.display(f, vars, name_gen)?;
        f.write_str(" -> ")?;
        self.ret.display(f, vars, name_gen)
    }
}

impl fmt::Display for VarTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "X{}", self.0)
    }
}
