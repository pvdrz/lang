use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    Lang,
    ir::{
        BinOp, DefId, Expr, ExprApp, ExprBinary, ExprCase, ExprFn, ExprIf, ExprLet, ExprUnary,
        Ident, Literal, LiteralKind, Pat, UnOp,
    },
    source_map::Span,
    ty::{FnTy, Ty, VarTy},
};

pub(crate) struct TyChecker<'ctx> {
    lang: &'ctx Lang,
    assumptions: HashMap<DefId, Ty>,
    constraints: Constraints<'ctx>,
    substitutions: Option<Substitutions>,
}

impl<'ctx> TyChecker<'ctx> {
    pub(crate) fn new(lang: &'ctx Lang) -> Self {
        Self {
            lang,
            assumptions: HashMap::new(),
            constraints: Constraints::new(lang),
            substitutions: None,
        }
    }

    fn add_assumption(&mut self, def_id: DefId, ty: Ty) {
        self.assumptions.insert(def_id, ty);

        print!("Adding assumption. Context now is {{");
        for (x, ty) in &self.assumptions {
            print!(" {}: {ty},", x.display(self.lang));
        }
        println!(" }}");
    }

    fn remove_assumption(&mut self, def_id: DefId) {
        self.assumptions.remove(&def_id);

        print!("Removing assumption. Context now is {{");
        for (x, ty) in &self.assumptions {
            print!(" {}: {ty},", x.display(self.lang));
        }
        println!(" }}");
    }

    pub(crate) fn infer_type(&mut self, expr: &mut Expr) -> Ty {
        let mut ty = self.type_expr(expr);
        let subs = self.constraints.unify();
        subs.substitute_ty(&mut ty);
        ty
    }

    fn type_expr(&mut self, expr: &Expr) -> Ty {
        let mut ty = match expr {
            Expr::Lit(literal) => self.type_literal(literal),
            Expr::Ident(ident) => self.type_ident(ident),
            Expr::Unary(expr_unary) => self.type_expr_unary(expr_unary),
            Expr::Binary(expr_binary) => self.type_expr_binary(expr_binary),
            Expr::If(expr_if) => self.type_expr_if(expr_if),
            Expr::Case(expr_case) => self.type_expr_case(expr_case),
            Expr::Let(expr_let) => self.type_expr_let(expr_let),
            Expr::Fn(expr_fn) => self.type_expr_fn(expr_fn),
            Expr::Apply(expr_app) => self.type_expr_app(expr_app),
        };

        if let Some(subs) = self.substitutions.as_ref() {
            subs.substitute_ty(&mut ty);
        }

        let before = ty.to_string();
        match &mut ty {
            Ty::ForAll(forall_ty) => {
                let span = expr.span();
                ty = forall_ty.instantiate(|| self.lang.gen_var_ty(span));
                println!("Instantiating {before} to {ty}");
            }
            _ => {}
        }

        ty
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
                self.constraints
                    .add(Ty::Int, expr_ty, expr_unary.expr.span());
                Ty::Int
            }
            UnOp::Not => {
                self.constraints
                    .add(Ty::Bool, expr_ty, expr_unary.expr.span());
                Ty::Bool
            }
        }
    }

    fn type_expr_binary(&mut self, expr_binary: &ExprBinary) -> Ty {
        let lhs_ty = self.type_expr(&expr_binary.lhs);
        let rhs_ty = self.type_expr(&expr_binary.rhs);
        self.constraints
            .add(lhs_ty.clone(), rhs_ty, expr_binary.span);

        match expr_binary.op {
            BinOp::Eq | BinOp::Ne => Ty::Bool,
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                self.constraints
                    .add(Ty::Int, lhs_ty.clone(), expr_binary.lhs.span());
                Ty::Bool
            }
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                self.constraints
                    .add(Ty::Int, lhs_ty.clone(), expr_binary.lhs.span());
                Ty::Int
            }
            BinOp::And | BinOp::Or => {
                self.constraints
                    .add(Ty::Bool, lhs_ty.clone(), expr_binary.lhs.span());
                Ty::Bool
            }
        }
    }

    fn type_expr_if(&mut self, expr_if: &ExprIf) -> Ty {
        let cond_ty = self.type_expr(&expr_if.cond);
        self.constraints.add(Ty::Bool, cond_ty, expr_if.cond.span());

        let do_ty = self.type_expr(&expr_if.do_branch);
        match expr_if.else_branch.as_deref() {
            Some(else_branch) => {
                let else_ty = self.type_expr(else_branch);
                self.constraints
                    .add(do_ty.clone(), else_ty, else_branch.span());
                do_ty
            }
            None => {
                self.constraints
                    .add(Ty::Unit, do_ty, expr_if.do_branch.span());
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
                self.constraints.add(ty.clone(), branch_ty, branch.span());
            }

            ty
        } else {
            self.constraints
                .add(Ty::Never, expr_ty, expr_case.expr.span());
            Ty::Never
        }
    }

    fn type_expr_let(&mut self, expr_let: &ExprLet) -> Ty {
        // We're typing `let lhs = rhs; body`
        println!("Typing let {} ..", expr_let.lhs.def_id.display(self.lang));

        let checkpoint = self.constraints.len();

        // Infer rhs: S.
        let mut rhs_ty = self.type_expr(&expr_let.rhs);

        // We only solve the constraints that involve the RHS.
        let mut constraints = self.constraints.checkpoint(checkpoint);
        println!("Unifying {} constraints", constraints.len());
        let subs = constraints.unify();
        println!("Done Unifying");

        // Now we apply the solution of the constraints to S to obtain a principal type T.
        subs.substitute_ty(&mut rhs_ty);

        let mut skip_var_tys = HashSet::new();

        for ty in self.assumptions.values() {
            ty.get_var_tys(&mut skip_var_tys);
        }

        // Generalize T to forall X1..Xn. T where none of the Xi is mentioned in the typing context.
        let before = rhs_ty.to_string();
        rhs_ty.generalize(|var_ty| {
            skip_var_tys.contains(&var_ty) && {
                println!("Skipping {var_ty} for generalization");
                true
            }
        });
        println!("Generalized {before} to {rhs_ty} ");

        self.add_assumption(expr_let.lhs.def_id, rhs_ty);
        self.substitutions = Some(subs);

        let body_ty = self.type_expr(&expr_let.body);

        self.substitutions = None;
        self.remove_assumption(expr_let.lhs.def_id);

        println!(
            "Done typing let {} .., type of body is: {body_ty}",
            expr_let.lhs.def_id.display(self.lang)
        );

        body_ty
    }

    fn type_expr_fn(&mut self, expr_fn: &ExprFn) -> Ty {
        let mut arg_tys = Vec::new();

        for arg in &expr_fn.args {
            let arg_ty = self.lang.gen_var_ty(arg.span);
            self.add_assumption(arg.def_id, arg_ty.clone());
            arg_tys.push(arg_ty);
        }

        let mut ty = self.type_expr(&expr_fn.body);

        for (arg, arg_ty) in expr_fn.args.iter().zip(arg_tys).rev() {
            self.remove_assumption(arg.def_id);
            ty = Ty::Fn(FnTy {
                arg: Box::new(arg_ty),
                ret: Box::new(ty),
            });
        }

        ty
    }

    fn type_expr_app(&mut self, expr_app: &ExprApp) -> Ty {
        let func_ty = self.type_expr(&expr_app.func);
        let arg_ty = self.type_expr(&expr_app.arg);
        let ret_ty = self.lang.gen_var_ty(expr_app.span);

        self.constraints.add(
            func_ty,
            Ty::Fn(FnTy {
                arg: Box::new(arg_ty),
                ret: Box::new(ret_ty.clone()),
            }),
            expr_app.span,
        );

        ret_ty
    }
}

struct Constraints<'ctx> {
    constraints: VecDeque<(Ty, Ty, Span)>,
    lang: &'ctx Lang,
}

impl<'ctx> Constraints<'ctx> {
    fn new(lang: &'ctx Lang) -> Self {
        Self {
            constraints: VecDeque::new(),
            lang,
        }
    }

    fn add(&mut self, lhs: Ty, rhs: Ty, span: Span) {
        let (line, col) = self.lang.source_map().map_offset(span.start());
        println!(
            "Adding constraint: {lhs} == {rhs} from {}:{}",
            line + 1,
            col + 1
        );
        self.constraints.push_back((lhs, rhs, span))
    }

    fn len(&self) -> usize {
        self.constraints.len()
    }

    fn checkpoint(&mut self, idx: usize) -> Self {
        Self {
            constraints: self.constraints.drain(idx..).collect(),
            lang: self.lang,
        }
    }

    fn replace(&mut self, var: VarTy, ty: &Ty) {
        for (lhs, rhs, _) in &mut self.constraints {
            lhs.replace(var, ty);
            rhs.replace(var, ty);
        }
    }

    fn unify(&mut self) -> Substitutions {
        let mut substitutions = HashMap::new();

        while let Some((lhs, rhs, span)) = self.constraints.pop_front() {
            if lhs == rhs {
                continue;
            } else if let Ty::Var(lhs) = lhs {
                let (line, col) = self.lang.source_map().map_offset(span.start());
                println!(
                    "Found substitution: {lhs} -> {rhs} from {}:{}",
                    line + 1,
                    col + 1
                );
                self.replace(lhs, &rhs);
                substitutions.insert(lhs, rhs);
            } else if let Ty::Var(rhs) = rhs {
                let (line, col) = self.lang.source_map().map_offset(span.start());
                println!(
                    "Found substitution: {rhs} -> {lhs} from {}:{}",
                    line + 1,
                    col + 1
                );
                self.replace(rhs, &lhs);
                substitutions.insert(rhs, lhs);
            } else {
                match (lhs, rhs) {
                    (Ty::Fn(lhs), Ty::Fn(rhs)) => {
                        self.add(*lhs.arg, *rhs.arg, span);
                        self.add(*lhs.ret, *rhs.ret, span);
                    }
                    (lhs, rhs) => {
                        self.lang
                            .error(span, format!("Expected type {lhs}, found {rhs}."));
                    }
                }
            }
        }

        Substitutions { substitutions }
    }
}

struct Substitutions {
    substitutions: HashMap<VarTy, Ty>,
}

impl Substitutions {
    // fn substitute_expr(&self, expr: &mut Expr) {
    //     match expr {
    //         Expr::Lit(_) | Expr::Ident(_) => (),
    //         Expr::Unary(expr_unary) => self.substitute_expr_unary(expr_unary),
    //         Expr::Binary(expr_binary) => self.substitute_expr_binary(expr_binary),
    //         Expr::If(expr_if) => self.substitute_expr_if(expr_if),
    //         Expr::Case(expr_case) => self.substitute_expr_case(expr_case),
    //         Expr::Let(expr_let) => self.substitute_expr_let(expr_let),
    //         Expr::Apply(expr_app) => self.substitute_expr_app(expr_app),
    //     }
    // }
    //
    // fn substitute_expr_unary(&self, expr_unary: &mut ExprUnary) {
    //     self.substitute_expr(&mut expr_unary.expr);
    // }
    //
    // fn substitute_expr_binary(&self, expr_binary: &mut ExprBinary) {
    //     self.substitute_expr(&mut expr_binary.lhs);
    //     self.substitute_expr(&mut expr_binary.rhs);
    // }
    //
    // fn substitute_expr_if(&self, expr_if: &mut ExprIf) {
    //     self.substitute_expr(&mut expr_if.cond);
    //     self.substitute_expr(&mut expr_if.do_branch);
    //     if let Some(else_branch) = expr_if.else_branch.as_deref_mut() {
    //         self.substitute_expr(else_branch);
    //     }
    // }
    //
    // fn substitute_expr_case(&self, expr_case: &mut ExprCase) {
    //     self.substitute_expr(&mut expr_case.expr);
    //
    //     for (_, expr) in &mut expr_case.arms {
    //         self.substitute_expr(expr);
    //     }
    // }
    //
    // fn substitute_expr_let(&self, expr_let: &mut ExprLet) {
    //     self.substitute_expr(&mut expr_let.rhs);
    //     self.substitute_expr(&mut expr_let.body);
    // }
    //
    // fn substitute_expr_app(&self, expr_app: &mut ExprApp) {
    //     self.substitute_expr(&mut expr_app.func);
    //     self.substitute_expr(&mut expr_app.arg);
    // }

    fn substitute_ty(&self, ty: &mut Ty) {
        match ty {
            Ty::Skolem(_) | Ty::Int | Ty::Float | Ty::String | Ty::Bool | Ty::Unit | Ty::Never => {}
            Ty::ForAll(forall_ty) => self.substitute_ty(&mut forall_ty.ty),
            Ty::Var(var) => {
                if let Some(subs) = self.substitutions.get(var) {
                    *ty = subs.clone();
                    self.substitute_ty(ty);
                }
            }
            Ty::Fn(fn_ty) => {
                self.substitute_ty(&mut fn_ty.ret);
                self.substitute_ty(&mut fn_ty.arg);
            }
        }
    }
}
