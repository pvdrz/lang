mod display;
mod replace;

use crate::def_gen;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Ty {
    ForAll(ForAllTy),
    Skolem(SkolemTy),
    Fn(FnTy),
    Var(VarTy),
    Int,
    Float,
    String,
    Bool,
    Unit,
    Never,
}

impl Ty {
    fn generalize_aux(&mut self, skip: impl Fn(VarTy) -> bool, vars: &mut Vec<VarTy>) {
        match self {
            Ty::ForAll(for_all_ty) => for_all_ty.ty.generalize_aux(skip, vars),
            Ty::Fn(fn_ty) => {
                fn_ty.arg.generalize_aux(&skip, vars);
                fn_ty.ret.generalize_aux(skip, vars);
            }
            Ty::Var(var_ty) => {
                if !skip(*var_ty) {
                    let index = vars
                        .iter()
                        .enumerate()
                        .find_map(
                            |(index, var)| {
                                if var == var_ty { Some(index) } else { None }
                            },
                        )
                        .unwrap_or_else(|| {
                            let index = vars.len();
                            vars.push(*var_ty);
                            index
                        });

                    *self = Ty::Skolem(SkolemTy { index, debruijn: 0 });
                }
            }
            Ty::Skolem(_) | Ty::Int | Ty::Float | Ty::String | Ty::Bool | Ty::Unit | Ty::Never => {
                todo!()
            }
        }
    }

    pub(crate) fn generalize(&mut self, skip: impl Fn(VarTy) -> bool + Copy) {
        let mut vars = Vec::new();
        self.generalize_aux(skip, &mut vars);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FnTy {
    pub(crate) arg: Box<Ty>,
    pub(crate) ret: Box<Ty>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ForAllTy {
    pub(crate) args: usize,
    pub(crate) ty: Box<Ty>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SkolemTy {
    index: usize,
    debruijn: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct VarTy(usize);

def_gen!(VarTyGen => VarTy);
