use crate::ast::{
    Expr, ExprApp, ExprBinary, ExprCase, ExprGroup, ExprIf, ExprLet, ExprUnary, Ident, Literal,
    Pat,
    op::{BinOp, UnOp},
};

pub(crate) trait Visitor {
    type Err;

    fn visit_ident(&mut self, ident: &Ident) -> Result<(), Self::Err> {
        visit_ident(self, ident)
    }

    fn visit_pat(&mut self, pat: &Pat) -> Result<(), Self::Err> {
        visit_pat(self, pat)
    }

    fn visit_expr(&mut self, expr: &Expr) -> Result<(), Self::Err> {
        visit_expr(self, expr)
    }

    fn visit_literal(&mut self, literal: &Literal) -> Result<(), Self::Err> {
        visit_literal(self, literal)
    }

    fn visit_expr_unary(&mut self, expr_unary: &ExprUnary) -> Result<(), Self::Err> {
        visit_expr_unary(self, expr_unary)
    }

    fn visit_un_op(&mut self, un_op: &UnOp) -> Result<(), Self::Err> {
        visit_un_op(self, un_op)
    }

    fn visit_expr_binary(&mut self, expr_binary: &ExprBinary) -> Result<(), Self::Err> {
        visit_expr_binary(self, expr_binary)
    }

    fn visit_bin_op(&mut self, bin_op: &BinOp) -> Result<(), Self::Err> {
        visit_bin_op(self, bin_op)
    }

    fn visit_expr_group(&mut self, expr_group: &ExprGroup) -> Result<(), Self::Err> {
        visit_expr_group(self, expr_group)
    }

    fn visit_expr_if(&mut self, expr_if: &ExprIf) -> Result<(), Self::Err> {
        visit_expr_if(self, expr_if)
    }

    fn visit_expr_case(&mut self, expr_case: &ExprCase) -> Result<(), Self::Err> {
        visit_expr_case(self, expr_case)
    }

    fn visit_expr_let(&mut self, expr_let: &ExprLet) -> Result<(), Self::Err> {
        visit_expr_let(self, expr_let)
    }

    fn visit_expr_app(&mut self, expr_app: &ExprApp) -> Result<(), Self::Err> {
        visit_expr_app(self, expr_app)
    }
}

pub(crate) fn visit_ident<V: Visitor + ?Sized>(_v: &mut V, _ident: &Ident) -> Result<(), V::Err> {
    Ok(())
}

pub(crate) fn visit_pat<V: Visitor + ?Sized>(v: &mut V, pat: &Pat) -> Result<(), V::Err> {
    match pat {
        Pat::Lit(expr_lit) => v.visit_literal(expr_lit),
        Pat::Ident(ident) => v.visit_ident(ident),
    }
}

pub(crate) fn visit_expr<V: Visitor + ?Sized>(v: &mut V, expr: &Expr) -> Result<(), V::Err> {
    match expr {
        Expr::Lit(literal) => v.visit_literal(literal),
        Expr::Ident(ident) => v.visit_ident(ident),
        Expr::Unary(expr_unary) => v.visit_expr_unary(expr_unary),
        Expr::Binary(expr_binary) => v.visit_expr_binary(expr_binary),
        Expr::Group(expr_group) => v.visit_expr_group(expr_group),
        Expr::If(expr_if) => v.visit_expr_if(expr_if),
        Expr::Case(expr_case) => v.visit_expr_case(expr_case),
        Expr::Let(expr_let) => v.visit_expr_let(expr_let),
        Expr::Apply(expr_app) => v.visit_expr_app(expr_app),
    }
}

pub(crate) fn visit_literal<V: Visitor + ?Sized>(
    _v: &mut V,
    _literal: &Literal,
) -> Result<(), V::Err> {
    Ok(())
}

pub(crate) fn visit_expr_unary<V: Visitor + ?Sized>(
    v: &mut V,
    expr_unary: &ExprUnary,
) -> Result<(), V::Err> {
    v.visit_un_op(&expr_unary.op)?;
    v.visit_expr(&expr_unary.expr)
}

pub(crate) fn visit_un_op<V: Visitor + ?Sized>(_v: &mut V, _un_op: &UnOp) -> Result<(), V::Err> {
    Ok(())
}

pub(crate) fn visit_expr_binary<V: Visitor + ?Sized>(
    v: &mut V,
    expr_binary: &ExprBinary,
) -> Result<(), V::Err> {
    v.visit_expr(&expr_binary.lhs)?;
    v.visit_bin_op(&expr_binary.op)?;
    v.visit_expr(&expr_binary.rhs)
}

pub(crate) fn visit_bin_op<V: Visitor + ?Sized>(_v: &mut V, _bin_op: &BinOp) -> Result<(), V::Err> {
    Ok(())
}

pub(crate) fn visit_expr_group<V: Visitor + ?Sized>(
    v: &mut V,
    expr_group: &ExprGroup,
) -> Result<(), V::Err> {
    v.visit_expr(&expr_group.expr)
}

pub(crate) fn visit_expr_if<V: Visitor + ?Sized>(
    v: &mut V,
    expr_if: &ExprIf,
) -> Result<(), V::Err> {
    v.visit_expr(&expr_if.cond)?;
    v.visit_expr(&expr_if.do_branch)?;
    if let Some(expr) = expr_if.else_branch.as_ref() {
        v.visit_expr(expr)?;
    }

    Ok(())
}

pub(crate) fn visit_expr_case<V: Visitor + ?Sized>(
    v: &mut V,
    expr_case: &ExprCase,
) -> Result<(), V::Err> {
    v.visit_expr(&expr_case.expr)?;

    for (pat, expr) in &expr_case.arms {
        v.visit_pat(pat)?;
        v.visit_expr(expr)?;
    }

    Ok(())
}

pub(crate) fn visit_expr_app<V: Visitor + ?Sized>(
    v: &mut V,
    expr_app: &ExprApp,
) -> Result<(), V::Err> {
    v.visit_expr(&expr_app.func)?;
    v.visit_expr(&expr_app.arg)
}

pub(crate) fn visit_expr_let<V: Visitor + ?Sized>(
    v: &mut V,
    expr_let: &ExprLet,
) -> Result<(), V::Err> {
    v.visit_ident(&expr_let.lhs)?;
    for ident in &expr_let.args {
        v.visit_ident(ident)?;
    }
    v.visit_expr(&expr_let.rhs)?;
    v.visit_expr(&expr_let.body)
}
