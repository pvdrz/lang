mod display;

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
    fn replace_aux(&mut self, var: VarTy, ty: &Ty, skip: impl Fn(VarTy) -> bool + Copy) {
        match self {
            Ty::ForAll(forall_ty) => forall_ty.replace_aux(var, ty, skip),
            Ty::Fn(fn_ty) => fn_ty.replace_aux(var, ty, skip),
            Ty::Var(x) => {
                if !skip(*x) && *x == var {
                    *self = ty.clone();
                }
            }
            Ty::Skolem(_) | Ty::Int | Ty::Float | Ty::String | Ty::Bool | Ty::Unit | Ty::Never => {}
        }
    }

    pub(crate) fn replace(&mut self, var: VarTy, ty: &Ty) {
        self.replace_aux(var, ty, |_| false);
    }

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

impl FnTy {
    fn replace_aux(&mut self, var: VarTy, ty: &Ty, skip: impl Fn(VarTy) -> bool + Copy) {
        self.arg.replace_aux(var, ty, skip);
        self.ret.replace_aux(var, ty, skip);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ForAllTy {
    pub(crate) args: usize,
    pub(crate) ty: Box<Ty>,
}

impl ForAllTy {
    fn replace_aux(&mut self, var: VarTy, ty: &Ty, skip: impl Fn(VarTy) -> bool + Copy) {
        self.ty.replace_aux(var, ty, skip);
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
