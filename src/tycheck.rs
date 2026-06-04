use std::collections::HashMap;

use crate::{
    Lang,
    ir::{
        DefId, Expr, ExprApp, ExprBinary, ExprCase, ExprIf, ExprLet, ExprUnary, Literal, Pat, UnOp,
    },
    ty::mono::{FnMonoTy, MonoTy, VarMonoTy, VarMonoTyGen},
};

pub(crate) struct TyChecker<'ctx> {
    lang: &'ctx Lang,
    assumptions: HashMap<DefId, MonoTy>,
    constraints: Vec<(MonoTy, MonoTy)>,
    substitutions: HashMap<VarMonoTy, MonoTy>,
    var_ty_gen: &'ctx mut VarMonoTyGen,
}

impl<'ctx> TyChecker<'ctx> {
    pub(crate) fn new(lang: &'ctx Lang, var_ty_gen: &'ctx mut VarMonoTyGen) -> Self {
        Self {
            lang,
            assumptions: HashMap::new(),
            constraints: Vec::new(),
            substitutions: HashMap::new(),
            var_ty_gen,
        }
    }

    pub(crate) fn infer_type(&mut self, expr: &mut Expr<MonoTy>) -> MonoTy {
        let mut ty = self.type_expr(expr);
        self.unify();
        self.substitute_expr(expr);
        self.substitute_ty(&mut ty);
        ty
    }

    fn type_expr(&mut self, expr: &Expr<MonoTy>) -> MonoTy {
        match expr {
            Expr::Lit(literal) => self.type_literal(literal),
            Expr::Ident(def_id) => self.type_def_id(def_id),
            Expr::Unary(expr_unary) => self.type_expr_unary(expr_unary),
            Expr::Binary(expr_binary) => self.type_expr_binary(expr_binary),
            Expr::If(expr_if) => self.type_expr_if(expr_if),
            Expr::Case(expr_case) => self.type_expr_case(expr_case),
            Expr::Let(expr_let) => self.type_expr_let(expr_let),
            Expr::Apply(expr_app) => self.type_expr_app(expr_app),
        }
    }

    fn type_literal(&mut self, literal: &Literal) -> MonoTy {
        match literal {
            Literal::Int(_) => MonoTy::Int,
            Literal::Float(_) => MonoTy::Float,
            Literal::Str(_) => MonoTy::String,
            Literal::True | Literal::False => MonoTy::Bool,
        }
    }

    fn type_def_id(&mut self, def_id: &DefId) -> MonoTy {
        self.assumptions[def_id].clone()
    }

    fn type_expr_unary(&mut self, expr_unary: &ExprUnary<MonoTy>) -> MonoTy {
        let expr_ty = self.type_expr(&expr_unary.expr);
        match expr_unary.op {
            UnOp::Neg => expr_ty,
            UnOp::Not => {
                self.constraints.push((MonoTy::Bool, expr_ty));
                MonoTy::Bool
            }
        }
    }

    fn type_expr_binary(&mut self, expr_binary: &ExprBinary<MonoTy>) -> MonoTy {
        let lhs_ty = self.type_expr(&expr_binary.lhs);
        let rhs_ty = self.type_expr(&expr_binary.rhs);
        self.constraints.push((lhs_ty.clone(), rhs_ty));
        lhs_ty
    }

    fn type_expr_if(&mut self, expr_if: &ExprIf<MonoTy>) -> MonoTy {
        let cond_ty = self.type_expr(&expr_if.cond);
        self.constraints.push((MonoTy::Bool, cond_ty));

        let do_ty = self.type_expr(&expr_if.do_branch);
        match expr_if.else_branch.as_deref() {
            Some(else_branch) => {
                let else_ty = self.type_expr(else_branch);
                self.constraints.push((do_ty.clone(), else_ty));
                do_ty
            }
            None => {
                self.constraints.push((MonoTy::Unit, do_ty));
                MonoTy::Unit
            }
        }
    }

    fn type_expr_case(&mut self, expr_case: &ExprCase<MonoTy>) -> MonoTy {
        let expr_ty = self.type_expr(&expr_case.expr);

        let mut branch_tys = Vec::new();

        for (pat, branch) in &expr_case.arms {
            let branch_ty = if let Pat::Ident(def_id) = pat {
                self.assumptions.insert(*def_id, expr_ty.clone());
                let branch_ty = self.type_expr(branch);
                self.assumptions.remove(def_id);
                branch_ty
            } else {
                self.type_expr(branch)
            };

            branch_tys.push(branch_ty);
        }

        if let Some(ty) = branch_tys.pop() {
            while let Some(branch_ty) = branch_tys.pop() {
                self.constraints.push((ty.clone(), branch_ty));
            }

            ty
        } else {
            MonoTy::Never
        }
    }

    fn type_expr_let(&mut self, expr_let: &ExprLet<MonoTy>) -> MonoTy {
        let mut lhs_ty = expr_let.ret_ty.clone();

        for (arg, arg_ty) in expr_let.args.iter().rev() {
            self.assumptions.insert(*arg, arg_ty.clone());
            lhs_ty = MonoTy::Fn(FnMonoTy {
                arg: Box::new(arg_ty.clone()),
                ret: Box::new(lhs_ty),
            });
        }

        let ret_ty = self.type_expr(&expr_let.rhs);
        self.constraints.push((expr_let.ret_ty.clone(), ret_ty));

        for (arg, _) in expr_let.args.iter() {
            self.assumptions.remove(arg);
        }

        self.assumptions.insert(expr_let.lhs, lhs_ty);
        let body_ty = self.type_expr(&expr_let.body);
        self.assumptions.remove(&expr_let.lhs);

        body_ty
    }

    fn type_expr_app(&mut self, expr_app: &ExprApp<MonoTy>) -> MonoTy {
        let func_ty = self.type_expr(&expr_app.func);
        let arg_ty = self.type_expr(&expr_app.arg);
        let ret_ty = MonoTy::Var(self.var_ty_gen.generate());

        self.constraints.push((
            func_ty,
            MonoTy::Fn(FnMonoTy {
                arg: Box::new(arg_ty),
                ret: Box::new(ret_ty.clone()),
            }),
        ));

        ret_ty
    }

    fn unify(&mut self) {
        while let Some((ty1, ty2)) = self.constraints.pop() {
            if ty1 == ty2 {
                continue;
            } else if let MonoTy::Var(x) = ty1 {
                self.replace(x, &ty2);
                self.substitutions.insert(x, ty2);
            } else if let MonoTy::Var(x) = ty2 {
                self.replace(x, &ty1);
                self.substitutions.insert(x, ty1);
            } else {
                match (ty1, ty2) {
                    (MonoTy::Fn(ty1), MonoTy::Fn(ty2)) => {
                        self.constraints.push((*ty1.arg, *ty2.arg));
                        self.constraints.push((*ty1.ret, *ty2.ret));
                    }
                    (ty1, ty2) => {
                        // FIXME: we need line information here.
                        self.lang
                            .error(0, format!("Cannot unify {ty1} with {ty2}."));
                    }
                }
            }
        }
    }

    fn replace(&mut self, var: VarMonoTy, ty: &MonoTy) {
        for (lhs, rhs) in &mut self.constraints {
            replace(lhs, var, ty);
            replace(rhs, var, ty);
        }
    }

    fn substitute_expr(&self, expr: &mut Expr<MonoTy>) {
        match expr {
            Expr::Lit(_) | Expr::Ident(_) => (),
            Expr::Unary(expr_unary) => self.substitute_expr_unary(expr_unary),
            Expr::Binary(expr_binary) => self.substitute_expr_binary(expr_binary),
            Expr::If(expr_if) => self.substitute_expr_if(expr_if),
            Expr::Case(expr_case) => self.substitute_expr_case(expr_case),
            Expr::Let(expr_let) => self.substitute_expr_let(expr_let),
            Expr::Apply(expr_app) => self.substitute_expr_app(expr_app),
        }
    }

    fn substitute_expr_unary(&self, expr_unary: &mut ExprUnary<MonoTy>) {
        self.substitute_expr(&mut expr_unary.expr);
    }

    fn substitute_expr_binary(&self, expr_binary: &mut ExprBinary<MonoTy>) {
        self.substitute_expr(&mut expr_binary.lhs);
        self.substitute_expr(&mut expr_binary.rhs);
    }

    fn substitute_expr_if(&self, expr_if: &mut ExprIf<MonoTy>) {
        self.substitute_expr(&mut expr_if.cond);
        self.substitute_expr(&mut expr_if.do_branch);
        if let Some(else_branch) = expr_if.else_branch.as_deref_mut() {
            self.substitute_expr(else_branch);
        }
    }

    fn substitute_expr_case(&self, expr_case: &mut ExprCase<MonoTy>) {
        self.substitute_expr(&mut expr_case.expr);

        for (_, expr) in &mut expr_case.arms {
            self.substitute_expr(expr);
        }
    }

    fn substitute_expr_let(&self, expr_let: &mut ExprLet<MonoTy>) {
        self.substitute_ty(&mut expr_let.ret_ty);
        self.substitute_expr(&mut expr_let.rhs);
        self.substitute_expr(&mut expr_let.body);
    }

    fn substitute_expr_app(&self, expr_app: &mut ExprApp<MonoTy>) {
        self.substitute_expr(&mut expr_app.func);
        self.substitute_expr(&mut expr_app.arg);
    }

    fn substitute_ty(&self, ty: &mut MonoTy) {
        match ty {
            MonoTy::Int
            | MonoTy::Float
            | MonoTy::String
            | MonoTy::Bool
            | MonoTy::Unit
            | MonoTy::Never => (),
            MonoTy::Var(var) => match self.substitutions.get(var) {
                Some(subs) => {
                    *ty = subs.clone();
                    self.substitute_ty(ty);
                }
                None => {
                    // FIXME: we need line information here
                    self.lang.error(0, format!("Cannot resolve type {var}"));
                }
            },
            MonoTy::Fn(fn_ty) => {
                self.substitute_ty(&mut fn_ty.ret);
                self.substitute_ty(&mut fn_ty.arg);
            }
        }
    }
}

fn replace(target: &mut MonoTy, var: VarMonoTy, ty: &MonoTy) {
    match target {
        MonoTy::Var(x) => {
            if *x == var {
                *target = ty.clone();
            }
        }
        MonoTy::Fn(fn_ty) => {
            replace(&mut fn_ty.arg, var, ty);
            replace(&mut fn_ty.ret, var, ty);
        }
        _ => (),
    }
}
