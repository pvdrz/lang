use super::*;

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
}

impl FnTy {
    fn replace_aux(&mut self, var: VarTy, ty: &Ty, skip: impl Fn(VarTy) -> bool + Copy) {
        self.arg.replace_aux(var, ty, skip);
        self.ret.replace_aux(var, ty, skip);
    }
}

impl ForAllTy {
    fn replace_aux(&mut self, var: VarTy, ty: &Ty, skip: impl Fn(VarTy) -> bool + Copy) {
        self.ty.replace_aux(var, ty, skip);
    }
}
