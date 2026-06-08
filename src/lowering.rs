use std::collections::{HashMap, hash_map::Entry};

use crate::{
    Lang,
    ast::{self},
    ir::{self, DefId},
};

pub(crate) struct Resolver<'ctx> {
    lang: &'ctx Lang,
    scope: Scope,
    scopes: Vec<Scope>,
}

#[derive(Default)]
struct Scope {
    inner: HashMap<ast::Ident, ir::Ident>,
}

impl<'ctx> Resolver<'ctx> {
    pub(crate) fn new(lang: &'ctx Lang) -> Self {
        Self {
            lang,
            scope: Scope::default(),
            scopes: Vec::new(),
        }
    }

    fn bind(&mut self, ident: &ast::Ident) -> ir::Ident {
        let mut lowered_ident = self.lang.gen_ident(ident);

        match self.scope.inner.entry(ident.clone()) {
            // This means we're shadowing the binding
            Entry::Occupied(mut entry) => std::mem::swap(entry.get_mut(), &mut lowered_ident),
            Entry::Vacant(entry) => {
                entry.insert(lowered_ident);
            }
        }

        lowered_ident
    }

    fn resolve(&self, ident: &ast::Ident) -> ir::Ident {
        for scope in [&self.scope].into_iter().chain(&self.scopes) {
            if let Some(lowered) = scope.inner.get(ident).copied() {
                return lowered;
            }
        }

        // We send the error but return an almost impossible to obtain ID so we can keep lowering
        // the AST and probably find more errors
        self.lang.error(
            ident.span(),
            format!("Cannot resolve identifier `{ident}`."),
        );
        ir::Ident {
            def_id: DefId::RIDICULOUS,
            span: ident.span(),
        }
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
    fn lower_ident(&mut self, ident: &ast::Ident) -> ir::Ident {
        self.resolve(ident)
    }

    fn lower_pat(&mut self, pat: &ast::Pat) -> ir::Pat {
        match pat {
            ast::Pat::Lit(literal) => ir::Pat::Lit(self.lower_literal(literal)),
            ast::Pat::Ident(ident) => ir::Pat::Ident(self.bind(ident)),
        }
    }

    pub(crate) fn lower_expr(&mut self, expr: &ast::Expr) -> ir::Expr {
        match expr {
            ast::Expr::Lit(literal) => ir::Expr::Lit(self.lower_literal(literal)),
            ast::Expr::Ident(ident) => ir::Expr::Ident(self.lower_ident(ident)),
            ast::Expr::Unary(expr_unary) => ir::Expr::Unary(self.lower_expr_unary(expr_unary)),
            ast::Expr::Binary(expr_binary) => ir::Expr::Binary(self.lower_expr_binary(expr_binary)),
            ast::Expr::Group(expr_group) => self.lower_expr_group(expr_group),
            ast::Expr::If(expr_if) => ir::Expr::If(self.lower_expr_if(expr_if)),
            ast::Expr::Case(expr_case) => ir::Expr::Case(self.lower_expr_case(expr_case)),
            ast::Expr::Let(expr_let) => ir::Expr::Let(self.lower_expr_let(expr_let)),
            ast::Expr::Apply(expr_app) => ir::Expr::Apply(self.lower_expr_app(expr_app)),
        }
    }

    fn lower_literal(&mut self, literal: &ast::Literal) -> ir::Literal {
        literal.clone()
    }

    fn lower_expr_unary(&mut self, expr_unary: &ast::ExprUnary) -> ir::ExprUnary {
        let op = self.lower_un_op(&expr_unary.op);
        let expr = self.lower_expr(&expr_unary.expr);

        ir::ExprUnary {
            op,
            expr: Box::new(expr),
            span: expr_unary.span,
        }
    }

    fn lower_un_op(&mut self, un_op: &ast::UnOp) -> ir::UnOp {
        *un_op
    }

    fn lower_expr_binary(&mut self, expr_binary: &ast::ExprBinary) -> ir::ExprBinary {
        let lhs = self.lower_expr(&expr_binary.lhs);
        let op = self.lower_bin_op(&expr_binary.op);
        let rhs = self.lower_expr(&expr_binary.rhs);

        ir::ExprBinary {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
            span: expr_binary.span,
        }
    }

    fn lower_bin_op(&mut self, bin_op: &ast::BinOp) -> ir::BinOp {
        *bin_op
    }

    fn lower_expr_group(&mut self, expr_group: &ast::ExprGroup) -> ir::Expr {
        self.lower_expr(&expr_group.expr)
    }

    fn lower_expr_if(&mut self, expr_if: &ast::ExprIf) -> ir::ExprIf {
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
            span: expr_if.span,
        }
    }

    fn lower_expr_case(&mut self, expr_case: &ast::ExprCase) -> ir::ExprCase {
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
            span: expr_case.span,
        }
    }

    fn lower_expr_let(&mut self, expr_let: &ast::ExprLet) -> ir::ExprLet {
        self.enter_scope();

        let lhs = self.bind(&expr_let.lhs);
        let args: Vec<_> = expr_let.args.iter().map(|arg| self.bind(arg)).collect();
        let rhs = self.lower_expr(&expr_let.rhs);
        let body = self.lower_expr(&expr_let.body);

        self.exit_scope();

        if args.is_empty() {
            ir::ExprLet {
                lhs,
                rhs: Box::new(rhs),
                body: Box::new(body),
                span: expr_let.span,
            }
        } else {
            ir::ExprLet {
                lhs,
                rhs: Box::new(ir::Expr::Fn(ir::ExprFn {
                    args,
                    body: Box::new(rhs),
                    span: expr_let.rhs.span(),
                })),
                body: Box::new(body),
                span: expr_let.span,
            }
        }
    }

    fn lower_expr_app(&mut self, expr_app: &ast::ExprApp) -> ir::ExprApp {
        let func = self.lower_expr(&expr_app.func);
        let arg = self.lower_expr(&expr_app.arg);

        ir::ExprApp {
            func: Box::new(func),
            arg: Box::new(arg),
            span: expr_app.span,
        }
    }
}
