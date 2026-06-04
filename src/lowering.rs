use std::collections::{HashMap, hash_map::Entry};

use crate::{
    Lang,
    ast::{self, Ident},
    ir::{self, DefId, DefIdGen},
    ty::mono::{MonoTy, MonoVarGen},
};

pub(crate) struct Resolver<'ctx> {
    lang: &'ctx Lang,
    scope: Scope,
    scopes: Vec<Scope>,
    def_id_gen: DefIdGen,
    mono_var_gen: &'ctx mut MonoVarGen,
}

#[derive(Default)]
struct Scope {
    inner: HashMap<Ident, ir::DefId>,
}

impl<'ctx> Resolver<'ctx> {
    pub(crate) fn new(lang: &'ctx Lang, mono_var_gen: &'ctx mut MonoVarGen) -> Self {
        Self {
            lang,
            scope: Scope::default(),
            scopes: Vec::new(),
            def_id_gen: DefIdGen::new(),
            mono_var_gen,
        }
    }

    fn bind(&mut self, ident: &Ident) -> DefId {
        let mut def_id = self.def_id_gen.generate();

        match self.scope.inner.entry(ident.clone()) {
            // This means we're shadowing the binding
            Entry::Occupied(mut entry) => std::mem::swap(entry.get_mut(), &mut def_id),
            Entry::Vacant(entry) => {
                entry.insert(def_id);
            }
        }

        def_id
    }

    fn resolve(&self, ident: &Ident) -> DefId {
        for scope in [&self.scope].into_iter().chain(&self.scopes) {
            if let Some(def_id) = scope.inner.get(ident).copied() {
                return def_id;
            }
        }

        // We send the error but return an almost impossible to obtain ID so we can keep lowering
        // the AST and probably find more errors
        self.lang.error(
            ident.line(),
            format!("Cannot resolve identifier `{ident}`."),
        );
        DefId::RIDICULOUS
    }

    fn enter_scope(&mut self) {
        let scope = std::mem::replace(&mut self.scope, Scope::default());
        self.scopes.push(scope);
    }

    fn exit_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            self.scope = scope;
        } else {
            unreachable!()
        }
    }
}

impl<'ctx> Resolver<'ctx> {
    fn lower_ident(&mut self, ident: &Ident) -> DefId {
        self.resolve(ident)
    }

    fn lower_pat(&mut self, pat: &ast::Pat) -> ir::Pat {
        match pat {
            ast::Pat::Lit(literal) => ir::Pat::Lit(self.lower_literal(literal)),
            ast::Pat::Ident(ident) => ir::Pat::Ident(self.bind(ident)),
        }
    }

    fn lower_let_binding(&mut self, binding: &ast::LetBinding) -> ir::LetBinding<MonoTy> {
        let lhs = self.bind(&binding.lhs);
        let ret_ty = self.mono_var_gen.generate();
        let args = binding
            .args
            .iter()
            .map(|arg| (self.bind(arg), self.mono_var_gen.generate()))
            .collect();
        let rhs = self.lower_expr(&binding.rhs);

        ir::LetBinding {
            lhs,
            ret_ty,
            args,
            rhs: Box::new(rhs),
        }
    }

    pub(crate) fn lower_expr(&mut self, expr: &ast::Expr) -> ir::Expr<MonoTy> {
        match expr {
            ast::Expr::Lit(literal) => ir::Expr::Lit(self.lower_literal(literal)),
            ast::Expr::Ident(ident) => ir::Expr::Ident(self.lower_ident(ident)),
            ast::Expr::Unary(expr_unary) => ir::Expr::Unary(self.lower_expr_unary(expr_unary)),
            ast::Expr::Binary(expr_binary) => ir::Expr::Binary(self.lower_expr_binary(expr_binary)),
            ast::Expr::Group(expr_group) => ir::Expr::Group(self.lower_expr_group(expr_group)),
            ast::Expr::If(expr_if) => ir::Expr::If(self.lower_expr_if(expr_if)),
            ast::Expr::Case(expr_case) => ir::Expr::Case(self.lower_expr_case(expr_case)),
            ast::Expr::Let(expr_let) => ir::Expr::Let(self.lower_expr_let(expr_let)),
            ast::Expr::Apply(expr_app) => ir::Expr::Apply(self.lower_expr_app(expr_app)),
        }
    }

    fn lower_literal(&mut self, literal: &ast::Literal) -> ir::Literal {
        literal.clone()
    }

    fn lower_expr_unary(&mut self, expr_unary: &ast::ExprUnary) -> ir::ExprUnary<MonoTy> {
        let op = self.lower_un_op(&expr_unary.op);
        let expr = self.lower_expr(&expr_unary.expr);

        ir::ExprUnary {
            op,
            expr: Box::new(expr),
        }
    }

    fn lower_un_op(&mut self, un_op: &ast::UnOp) -> ir::UnOp {
        *un_op
    }

    fn lower_expr_binary(&mut self, expr_binary: &ast::ExprBinary) -> ir::ExprBinary<MonoTy> {
        let lhs = self.lower_expr(&expr_binary.lhs);
        let op = self.lower_bin_op(&expr_binary.op);
        let rhs = self.lower_expr(&expr_binary.rhs);

        ir::ExprBinary {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
        }
    }

    fn lower_bin_op(&mut self, bin_op: &ast::BinOp) -> ir::BinOp {
        *bin_op
    }

    fn lower_expr_group(&mut self, expr_group: &ast::ExprGroup) -> ir::ExprGroup<MonoTy> {
        let expr = self.lower_expr(&expr_group.expr);

        ir::ExprGroup {
            expr: Box::new(expr),
        }
    }

    fn lower_expr_if(&mut self, expr_if: &ast::ExprIf) -> ir::ExprIf<MonoTy> {
        let cond = self.lower_expr(&expr_if.cond);
        let do_branch = self.lower_expr(&expr_if.do_branch);
        let else_branch = expr_if
            .else_branch
            .as_ref()
            .map(|expr| Box::new(self.lower_expr(expr)));

        ir::ExprIf {
            cond: Box::new(cond),
            do_branch: Box::new(do_branch),
            else_branch,
        }
    }

    fn lower_expr_case(&mut self, expr_case: &ast::ExprCase) -> ir::ExprCase<MonoTy> {
        let expr = self.lower_expr(&expr_case.expr);
        let arms = expr_case
            .arms
            .iter()
            .map(|(pat, expr)| {
                self.enter_scope();
                let pat = self.lower_pat(pat);
                let expr = self.lower_expr(expr);
                self.exit_scope();

                (pat, expr)
            })
            .collect();

        ir::ExprCase {
            expr: Box::new(expr),
            arms,
        }
    }

    fn lower_expr_let(&mut self, expr_let: &ast::ExprLet) -> ir::ExprLet<MonoTy> {
        self.enter_scope();

        let binding = self.lower_let_binding(&expr_let.binding);
        let tail = self.lower_expr(&expr_let.tail);

        self.exit_scope();

        ir::ExprLet {
            binding,
            tail: Box::new(tail),
        }
    }

    fn lower_expr_app(&mut self, expr_app: &ast::ExprApp) -> ir::ExprApp<MonoTy> {
        let func = self.lower_expr(&expr_app.func);
        let arg = self.lower_expr(&expr_app.arg);

        ir::ExprApp {
            func: Box::new(func),
            arg: Box::new(arg),
        }
    }
}
