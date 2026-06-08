mod display;
mod replace;

use std::collections::HashSet;

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
    fn generalize_aux(&mut self, skip: impl Fn(VarTy) -> bool + Copy, vars: &mut Vec<VarTy>) {
        match self {
            Ty::ForAll(for_all_ty) => for_all_ty.ty.generalize_aux(skip, vars),
            Ty::Fn(fn_ty) => {
                fn_ty.arg.generalize_aux(skip, vars);
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
            Ty::Skolem(_) | Ty::Int | Ty::Float | Ty::String | Ty::Bool | Ty::Unit | Ty::Never => {}
        }
    }

    pub(crate) fn generalize(&mut self, skip: impl Fn(VarTy) -> bool + Copy) {
        let mut vars = Vec::new();
        self.generalize_aux(skip, &mut vars);
        if !vars.is_empty() {
            *self = Self::ForAll(ForAllTy {
                args: vars.len(),
                ty: Box::new(self.clone()),
            });
        }
    }

    pub(crate) fn get_var_tys(&self, var_tys: &mut HashSet<VarTy>) {
        match self {
            Ty::ForAll(for_all_ty) => for_all_ty.ty.get_var_tys(var_tys),
            Ty::Fn(fn_ty) => {
                fn_ty.arg.get_var_tys(var_tys);
                fn_ty.ret.get_var_tys(var_tys);
            }
            Ty::Var(var_ty) => {
                var_tys.insert(*var_ty);
            }
            Ty::Skolem(_) | Ty::Int | Ty::Float | Ty::String | Ty::Bool | Ty::Unit | Ty::Never => {}
        }
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

impl ForAllTy {
    pub(crate) fn instantiate(&self, generate_var: impl Fn() -> Ty + Copy) -> Ty {
        let mut ty = self.ty.as_ref().clone();
        for index in 0..self.args {
            let arg = generate_var();
            ty.replace_skolem(SkolemTy { index, debruijn: 0 }, &arg);
        }

        ty
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SkolemTy {
    index: usize,
    debruijn: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct VarTy(usize);

def_gen!(VarTyGen => VarTy);
