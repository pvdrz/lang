use std::fmt::Write;

use crate::ast::{BinOp, Expr, Literal, LiteralKind, UnOp, visitor::Visitor};

pub(crate) fn pretty_print(expr: &Expr) -> String {
    let mut pp = PrettyPrinter { buf: String::new() };
    pp.visit_expr(expr);
    pp.buf
}

struct PrettyPrinter {
    buf: String,
}

impl PrettyPrinter {
    fn parenthesize(&mut self, name: Option<&str>, f: impl for<'b> Fn(&mut Self)) {
        let _ = self.buf.write_str("(");
        if let Some(name) = name {
            let _ = write!(self.buf, "{name} ");
        }
        f(self);
        let _ = self.buf.write_str(")");
    }
}

impl Visitor for PrettyPrinter {
    fn visit_ident(&mut self, ident: &super::Ident) {
        let _ = self.buf.write_str(&ident.inner);
        super::visitor::visit_ident(self, ident);
    }

    fn visit_literal(&mut self, expr_lit: &Literal) {
        let _ = match &expr_lit.kind {
            LiteralKind::Int(int) => write!(self.buf, "{int}"),
            LiteralKind::Float(float) => write!(self.buf, "{float}"),
            LiteralKind::Str(string) => self.buf.write_str(string),
            LiteralKind::True => self.buf.write_str("true"),
            LiteralKind::False => self.buf.write_str("false"),
        };
    }

    fn visit_expr_case(&mut self, expr_case: &super::ExprCase) {
        let _ = self.buf.write_str("case ");

        self.visit_expr(&expr_case.expr);

        let _ = self.buf.write_str(" do ");

        for (pat, expr) in &expr_case.arms {
            self.visit_pat(pat);

            let _ = self.buf.write_str(" -> ");

            self.visit_expr(expr);

            let _ = self.buf.write_str(", ");
        }

        let _ = self.buf.write_str("end");
    }

    fn visit_expr_let(&mut self, expr_let: &super::ExprLet) {
        let _ = self.buf.write_str("let ");
        self.visit_ident(&expr_let.lhs);

        for ident in &expr_let.args {
            let _ = self.buf.write_str(" ");
            self.visit_ident(ident);
        }

        let _ = self.buf.write_str(" = (");
        self.visit_expr(&expr_let.rhs);
        let _ = self.buf.write_str(") ");
        self.visit_expr(&expr_let.body);
    }

    fn visit_expr_unary(&mut self, expr_unary: &super::ExprUnary) {
        self.parenthesize(None, |this| {
            super::visitor::visit_expr_unary(this, expr_unary);
        });
    }

    fn visit_un_op(&mut self, un_op: &UnOp) {
        let _ = match un_op {
            UnOp::Neg => self.buf.write_str("-"),
            UnOp::Not => self.buf.write_str("!"),
        };
    }

    fn visit_expr_binary(&mut self, expr_binary: &super::ExprBinary) {
        self.parenthesize(None, |this| {
            super::visitor::visit_expr_binary(this, expr_binary)
        })
    }

    fn visit_bin_op(&mut self, bin_op: &BinOp) {
        let _ = match bin_op {
            BinOp::Eq => self.buf.write_str(" == "),
            BinOp::Ne => self.buf.write_str(" != "),
            BinOp::Lt => self.buf.write_str(" < "),
            BinOp::Le => self.buf.write_str(" <= "),
            BinOp::Gt => self.buf.write_str(" > "),
            BinOp::Ge => self.buf.write_str(" >= "),
            BinOp::Add => self.buf.write_str(" + "),
            BinOp::Sub => self.buf.write_str(" - "),
            BinOp::Mul => self.buf.write_str(" * "),
            BinOp::Div => self.buf.write_str(" / "),
            BinOp::And => self.buf.write_str(" && "),
            BinOp::Or => self.buf.write_str(" || "),
        };
    }

    fn visit_expr_group(&mut self, expr_group: &super::ExprGroup) {
        self.parenthesize(Some("group"), |this| {
            super::visitor::visit_expr_group(this, expr_group)
        })
    }

    fn visit_expr_if(&mut self, expr_if: &super::ExprIf) {
        let _ = self.buf.write_str("if ");
        self.visit_expr(&expr_if.cond);

        let _ = self.buf.write_str(" do ");
        self.visit_expr(&expr_if.do_branch);

        if let Some(expr) = expr_if.else_branch.as_ref() {
            let _ = self.buf.write_str(" else ");
            self.visit_expr(expr);
        }
        let _ = self.buf.write_str(" end");
    }

    fn visit_expr_app(&mut self, expr_app: &super::ExprApp) {
        self.parenthesize(None, |this| {
            this.visit_expr(&expr_app.func);
            let _ = this.buf.write_str(" ");
            this.visit_expr(&expr_app.arg);
        });
    }
}
