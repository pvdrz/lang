use std::collections::{HashMap, VecDeque};

use crate::{
    Lang,
    ir::{
        BinOp, DefId, Expr, ExprApp, ExprBinary, ExprCase, ExprIf, ExprLet, ExprUnary, Ident,
        Literal, LiteralKind, Pat, UnOp,
    },
    source_map::Span,
    ty::{FnTy, Ty, VarTy},
};

pub(crate) struct TyChecker<'ctx> {
    lang: &'ctx Lang,
    assumptions: HashMap<DefId, Ty>,
    constraints: VecDeque<Constraint>,
    substitutions: HashMap<VarTy, Ty>,
}

struct Constraint {
    lhs: Ty,
    rhs: Ty,
    span: Span,
}

impl<'ctx> TyChecker<'ctx> {
    pub(crate) fn new(lang: &'ctx Lang) -> Self {
        Self {
            lang,
            assumptions: HashMap::new(),
            constraints: VecDeque::new(),
            substitutions: HashMap::new(),
        }
    }

    fn add_assumption(&mut self, def_id: DefId, ty: Ty) {
        println!("Adding assumption: {def_id:?}: {ty}");
        self.assumptions.insert(def_id, ty);
    }

    fn remove_assumption(&mut self, def_id: DefId) {
        println!("Removing assumption for {def_id:?}");
        self.assumptions.remove(&def_id);
    }

    pub(crate) fn infer_type(&mut self, expr: &mut Expr) -> Ty {
        let mut ty = self.type_expr(expr);
        self.unify();
        self.substitute_expr(expr);
        self.substitute_ty(&mut ty);
        ty
    }

    fn type_expr(&mut self, expr: &Expr) -> Ty {
        match expr {
            Expr::Lit(literal) => self.type_literal(literal),
            Expr::Ident(ident) => self.type_ident(ident),
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

    fn type_ident(&mut self, ident: &Ident) -> Ty {
        self.assumptions[&ident.def_id].clone()
    }

    fn type_expr_unary(&mut self, expr_unary: &ExprUnary) -> Ty {
        let expr_ty = self.type_expr(&expr_unary.expr);
        match expr_unary.op {
            UnOp::Neg => {
                self.add_constraint(Ty::Int, expr_ty, expr_unary.expr.span());
                Ty::Int
            }
            UnOp::Not => {
                self.add_constraint(Ty::Bool, expr_ty, expr_unary.expr.span());
                Ty::Bool
            }
        }
    }

    fn type_expr_binary(&mut self, expr_binary: &ExprBinary) -> Ty {
        let lhs_ty = self.type_expr(&expr_binary.lhs);
        let rhs_ty = self.type_expr(&expr_binary.rhs);
        self.add_constraint(lhs_ty.clone(), rhs_ty, expr_binary.span);

        match expr_binary.op {
            BinOp::Eq | BinOp::Ne => Ty::Bool,
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                self.add_constraint(Ty::Int, lhs_ty.clone(), expr_binary.lhs.span());
                Ty::Bool
            }
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                self.add_constraint(Ty::Int, lhs_ty.clone(), expr_binary.lhs.span());
                Ty::Int
            }
            BinOp::And | BinOp::Or => {
                self.add_constraint(Ty::Bool, lhs_ty.clone(), expr_binary.lhs.span());
                Ty::Bool
            }
            _ => lhs_ty,
        }
    }

    fn type_expr_if(&mut self, expr_if: &ExprIf) -> Ty {
        let cond_ty = self.type_expr(&expr_if.cond);
        self.add_constraint(Ty::Bool, cond_ty, expr_if.cond.span());

        let do_ty = self.type_expr(&expr_if.do_branch);
        match expr_if.else_branch.as_deref() {
            Some(else_branch) => {
                let else_ty = self.type_expr(else_branch);
                self.add_constraint(do_ty.clone(), else_ty, else_branch.span());
                do_ty
            }
            None => {
                self.add_constraint(Ty::Unit, do_ty, expr_if.do_branch.span());
                Ty::Unit
            }
        }
    }

    fn type_expr_case(&mut self, expr_case: &ExprCase) -> Ty {
        let expr_ty = self.type_expr(&expr_case.expr);

        let mut branch_tys = Vec::new();

        for (pat, branch) in &expr_case.arms {
            let branch_ty = if let Pat::Ident(ident) = pat {
                self.add_assumption(ident.def_id, expr_ty.clone());
                let branch_ty = self.type_expr(branch);
                self.remove_assumption(ident.def_id);
                branch_ty
            } else {
                self.type_expr(branch)
            };

            branch_tys.push(branch_ty);
        }

        if let Some(ty) = branch_tys.pop() {
            let mut arms = expr_case.arms.iter().skip(1);
            while let (Some(branch_ty), Some((_, branch))) = (branch_tys.pop(), arms.next()) {
                self.add_constraint(ty.clone(), branch_ty, branch.span());
            }

            ty
        } else {
            Ty::Never
        }
    }

    fn type_expr_let(&mut self, expr_let: &ExprLet) -> Ty {
        let ret_ty = self.lang.gen_var_ty(expr_let.lhs.span);
        let mut lhs_ty = ret_ty.clone();

        for arg in expr_let.args.iter().rev() {
            let arg_ty = self.lang.gen_var_ty(arg.span);
            self.add_assumption(arg.def_id, arg_ty.clone());
            lhs_ty = Ty::Fn(FnTy {
                arg: Box::new(arg_ty),
                ret: Box::new(lhs_ty),
            });
        }

        let infered_ret_ty = self.type_expr(&expr_let.rhs);
        self.add_constraint(ret_ty, infered_ret_ty, expr_let.lhs.span);

        for arg in expr_let.args.iter() {
            self.remove_assumption(arg.def_id);
        }

        self.add_assumption(expr_let.lhs.def_id, lhs_ty);
        let body_ty = self.type_expr(&expr_let.body);
        self.remove_assumption(expr_let.lhs.def_id);

        body_ty
    }

    fn type_expr_app(&mut self, expr_app: &ExprApp) -> Ty {
        let func_ty = self.type_expr(&expr_app.func);
        let arg_ty = self.type_expr(&expr_app.arg);
        let ret_ty = self.lang.gen_var_ty(expr_app.span);

        self.add_constraint(
            func_ty,
            Ty::Fn(FnTy {
                arg: Box::new(arg_ty),
                ret: Box::new(ret_ty.clone()),
            }),
            expr_app.span,
        );

        ret_ty
    }

    fn add_constraint(&mut self, lhs: Ty, rhs: Ty, span: Span) {
        let (line, col) = self.lang.source_map().map_offset(span.start());
        println!(
            "Adding constraint: {lhs} == {rhs} from {}:{}",
            line + 1,
            col + 1
        );
        self.constraints.push_back(Constraint { lhs, rhs, span })
    }

    fn unify(&mut self) {
        while let Some(Constraint { lhs, rhs, span }) = self.constraints.pop_front() {
            if lhs == rhs {
                continue;
            } else if let Ty::Var(x) = lhs {
                let (line, col) = self.lang.source_map().map_offset(span.start());
                println!(
                    "Unified constraint: {lhs} = {rhs} from {}:{}",
                    line + 1,
                    col + 1
                );
                self.replace(x, &rhs);
                self.substitutions.insert(x, rhs);
            } else if let Ty::Var(x) = rhs {
                let (line, col) = self.lang.source_map().map_offset(span.start());
                println!(
                    "Unified constraint: {lhs} = {rhs} from {}:{}",
                    line + 1,
                    col + 1
                );
                self.replace(x, &lhs);
                self.substitutions.insert(x, lhs);
            } else {
                match (lhs, rhs) {
                    (Ty::Fn(lhs), Ty::Fn(rhs)) => {
                        self.add_constraint(*lhs.arg, *rhs.arg, span);
                        self.add_constraint(*lhs.ret, *rhs.ret, span);
                    }
                    (lhs, rhs) => {
                        self.lang
                            .error(span, format!("Expected type {lhs}, found {rhs}."));
                    }
                }
            }
        }
    }

    fn replace(&mut self, var: VarTy, ty: &Ty) {
        for Constraint { lhs, rhs, .. } in &mut self.constraints {
            replace(lhs, var, ty);
            replace(rhs, var, ty);
        }
    }

    fn substitute_expr(&self, expr: &mut Expr) {
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

    fn substitute_expr_unary(&self, expr_unary: &mut ExprUnary) {
        self.substitute_expr(&mut expr_unary.expr);
    }

    fn substitute_expr_binary(&self, expr_binary: &mut ExprBinary) {
        self.substitute_expr(&mut expr_binary.lhs);
        self.substitute_expr(&mut expr_binary.rhs);
    }

    fn substitute_expr_if(&self, expr_if: &mut ExprIf) {
        self.substitute_expr(&mut expr_if.cond);
        self.substitute_expr(&mut expr_if.do_branch);
        if let Some(else_branch) = expr_if.else_branch.as_deref_mut() {
            self.substitute_expr(else_branch);
        }
    }

    fn substitute_expr_case(&self, expr_case: &mut ExprCase) {
        self.substitute_expr(&mut expr_case.expr);

        for (_, expr) in &mut expr_case.arms {
            self.substitute_expr(expr);
        }
    }

    fn substitute_expr_let(&self, expr_let: &mut ExprLet) {
        self.substitute_expr(&mut expr_let.rhs);
        self.substitute_expr(&mut expr_let.body);
    }

    fn substitute_expr_app(&self, expr_app: &mut ExprApp) {
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
                    self.lang
                        .error(self.lang.var_ty_span(*var), "Cannot resolve type.");
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
