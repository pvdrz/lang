use std::collections::HashMap;

use crate::{
    Lang,
    ir::{
        DefId, Expr, ExprApp, ExprBinary, ExprCase, ExprIf, ExprLet, ExprUnary, Literal,
        LiteralKind, Pat, UnOp,
    },
    source_map::Span,
    ty::{FnTy, Ty, VarTy},
};

pub(crate) struct TyChecker<'ctx> {
    lang: &'ctx Lang,
    assumptions: HashMap<DefId, Ty>,
    constraints: Vec<(Ty, Ty)>,
    substitutions: HashMap<VarTy, Ty>,
}

impl<'ctx> TyChecker<'ctx> {
    pub(crate) fn new(lang: &'ctx Lang) -> Self {
        Self {
            lang,
            assumptions: HashMap::new(),
            constraints: Vec::new(),
            substitutions: HashMap::new(),
        }
    }

    pub(crate) fn infer_type(&mut self, expr: &mut Expr<Ty>) -> Ty {
        let mut ty = self.type_expr(expr);
        self.unify();
        self.substitute_expr(expr);
        self.substitute_ty(&mut ty);
        ty
    }

    fn type_expr(&mut self, expr: &Expr<Ty>) -> Ty {
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

    fn type_literal(&mut self, literal: &Literal) -> Ty {
        match &literal.kind {
            LiteralKind::Int(_) => Ty::Int,
            LiteralKind::Float(_) => Ty::Float,
            LiteralKind::Str(_) => Ty::String,
            LiteralKind::True | LiteralKind::False => Ty::Bool,
        }
    }

    fn type_def_id(&mut self, def_id: &DefId) -> Ty {
        self.assumptions[def_id].clone()
    }

    fn type_expr_unary(&mut self, expr_unary: &ExprUnary<Ty>) -> Ty {
        let expr_ty = self.type_expr(&expr_unary.expr);
        match expr_unary.op {
            UnOp::Neg => {
                self.constraints.push((Ty::Int, expr_ty));
                Ty::Int
            }
            UnOp::Not => {
                self.constraints.push((Ty::Bool, expr_ty));
                Ty::Bool
            }
        }
    }

    fn type_expr_binary(&mut self, expr_binary: &ExprBinary<Ty>) -> Ty {
        let lhs_ty = self.type_expr(&expr_binary.lhs);
        let rhs_ty = self.type_expr(&expr_binary.rhs);
        self.constraints.push((lhs_ty.clone(), rhs_ty));

        match expr_binary.op {
            crate::ir::BinOp::Lt
            | crate::ir::BinOp::Le
            | crate::ir::BinOp::Gt
            | crate::ir::BinOp::Ge
            | crate::ir::BinOp::Add
            | crate::ir::BinOp::Sub
            | crate::ir::BinOp::Mul
            | crate::ir::BinOp::Div => {
                self.constraints.push((Ty::Int, lhs_ty.clone()));
                Ty::Int
            }
            crate::ir::BinOp::And | crate::ir::BinOp::Or => {
                self.constraints.push((Ty::Bool, lhs_ty.clone()));
                Ty::Bool
            }
            _ => lhs_ty,
        }
    }

    fn type_expr_if(&mut self, expr_if: &ExprIf<Ty>) -> Ty {
        let cond_ty = self.type_expr(&expr_if.cond);
        self.constraints.push((Ty::Bool, cond_ty));

        let do_ty = self.type_expr(&expr_if.do_branch);
        match expr_if.else_branch.as_deref() {
            Some(else_branch) => {
                let else_ty = self.type_expr(else_branch);
                self.constraints.push((do_ty.clone(), else_ty));
                do_ty
            }
            None => {
                self.constraints.push((Ty::Unit, do_ty));
                Ty::Unit
            }
        }
    }

    fn type_expr_case(&mut self, expr_case: &ExprCase<Ty>) -> Ty {
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
            Ty::Never
        }
    }

    fn type_expr_let(&mut self, expr_let: &ExprLet<Ty>) -> Ty {
        let mut lhs_ty = expr_let.ret_ty.clone();

        for (arg, arg_ty) in expr_let.args.iter().rev() {
            self.assumptions.insert(*arg, arg_ty.clone());
            lhs_ty = Ty::Fn(FnTy {
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

    fn type_expr_app(&mut self, expr_app: &ExprApp<Ty>) -> Ty {
        let func_ty = self.type_expr(&expr_app.func);
        let arg_ty = self.type_expr(&expr_app.arg);
        let ret_ty = self.lang.gen_var_ty();

        self.constraints.push((
            func_ty,
            Ty::Fn(FnTy {
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
            } else if let Ty::Var(x) = ty1 {
                self.replace(x, &ty2);
                self.substitutions.insert(x, ty2);
            } else if let Ty::Var(x) = ty2 {
                self.replace(x, &ty1);
                self.substitutions.insert(x, ty1);
            } else {
                match (ty1, ty2) {
                    (Ty::Fn(ty1), Ty::Fn(ty2)) => {
                        self.constraints.push((*ty1.arg, *ty2.arg));
                        self.constraints.push((*ty1.ret, *ty2.ret));
                    }
                    (ty1, ty2) => {
                        // FIXME: we need line information here.
                        self.lang
                            .error(Span::DUMMY, format!("Cannot unify {ty1} with {ty2}."));
                    }
                }
            }
        }
    }

    fn replace(&mut self, var: VarTy, ty: &Ty) {
        for (lhs, rhs) in &mut self.constraints {
            replace(lhs, var, ty);
            replace(rhs, var, ty);
        }
    }

    fn substitute_expr(&self, expr: &mut Expr<Ty>) {
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

    fn substitute_expr_unary(&self, expr_unary: &mut ExprUnary<Ty>) {
        self.substitute_expr(&mut expr_unary.expr);
    }

    fn substitute_expr_binary(&self, expr_binary: &mut ExprBinary<Ty>) {
        self.substitute_expr(&mut expr_binary.lhs);
        self.substitute_expr(&mut expr_binary.rhs);
    }

    fn substitute_expr_if(&self, expr_if: &mut ExprIf<Ty>) {
        self.substitute_expr(&mut expr_if.cond);
        self.substitute_expr(&mut expr_if.do_branch);
        if let Some(else_branch) = expr_if.else_branch.as_deref_mut() {
            self.substitute_expr(else_branch);
        }
    }

    fn substitute_expr_case(&self, expr_case: &mut ExprCase<Ty>) {
        self.substitute_expr(&mut expr_case.expr);

        for (_, expr) in &mut expr_case.arms {
            self.substitute_expr(expr);
        }
    }

    fn substitute_expr_let(&self, expr_let: &mut ExprLet<Ty>) {
        self.substitute_ty(&mut expr_let.ret_ty);
        self.substitute_expr(&mut expr_let.rhs);
        self.substitute_expr(&mut expr_let.body);
    }

    fn substitute_expr_app(&self, expr_app: &mut ExprApp<Ty>) {
        self.substitute_expr(&mut expr_app.func);
        self.substitute_expr(&mut expr_app.arg);
    }

    fn substitute_ty(&self, ty: &mut Ty) {
        match ty {
            Ty::Int | Ty::Float | Ty::String | Ty::Bool | Ty::Unit | Ty::Never => (),
            Ty::Var(var) => match self.substitutions.get(var) {
                Some(subs) => {
                    *ty = subs.clone();
                    self.substitute_ty(ty);
                }
                None => {
                    // FIXME: we need line information here
                    self.lang
                        .error(Span::DUMMY, format!("Cannot resolve type {var}"));
                }
            },
            Ty::Fn(fn_ty) => {
                self.substitute_ty(&mut fn_ty.ret);
                self.substitute_ty(&mut fn_ty.arg);
            }
        }
    }
}

fn replace(target: &mut Ty, var: VarTy, ty: &Ty) {
    match target {
        Ty::Var(x) => {
            if *x == var {
                *target = ty.clone();
            }
        }
        Ty::Fn(fn_ty) => {
            replace(&mut fn_ty.arg, var, ty);
            replace(&mut fn_ty.ret, var, ty);
        }
        _ => (),
    }
}
