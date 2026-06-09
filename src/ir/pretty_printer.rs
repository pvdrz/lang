use std::fmt;

use crate::{
    Lang,
    ir::{BinOp, Expr, Literal, LiteralKind, UnOp, visitor::Visitor},
};

pub(crate) fn pretty_print<'a>(lang: &'a Lang, expr: &'a Expr) -> impl fmt::Display + 'a {
    struct PrettyPrintable<'a> {
        lang: &'a Lang,
        expr: &'a Expr,
    };

    impl fmt::Display for PrettyPrintable<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut pp = PrettyPrinter {
                lang: self.lang,
                f,
                level: 0,
                after_newline: false,
            };
            pp.visit_expr(self.expr)
        }
    }

    PrettyPrintable { lang, expr }
}

struct PrettyPrinter<'ctx, 'a> {
    lang: &'ctx Lang,
    f: &'ctx mut fmt::Formatter<'a>,
    level: usize,
    after_newline: bool,
}

impl PrettyPrinter<'_, '_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.after_newline {
            write!(&mut self.f, "{:width$}", "", width = self.level)?;
            self.after_newline = false;
        }

        self.f.write_str(s)
    }

    fn parenthesize(&mut self, f: impl for<'b> Fn(&mut Self) -> fmt::Result) -> fmt::Result {
        self.write_str("(")?;
        f(self)?;
        self.write_str(")")
    }

    fn newline(&mut self) -> fmt::Result {
        self.f.write_str("\n")?;
        self.after_newline = true;
        Ok(())
    }

    fn indent(&mut self) {
        self.level += 2;
    }

    fn dedent(&mut self) {
        self.level -= 2;
    }
}

impl Visitor for PrettyPrinter<'_, '_> {
    type Err = fmt::Error;

    fn visit_ident(&mut self, ident: &super::Ident) -> fmt::Result {
        self.write_str("")?;
        write!(self.f, "{}", ident.def_id.display(self.lang))?;

        super::visitor::visit_ident(self, ident)
    }

    fn visit_literal(&mut self, expr_lit: &Literal) -> fmt::Result {
        match &expr_lit.kind {
            LiteralKind::Int(int) => write!(self.f, "{int}"),
            LiteralKind::Float(float) => write!(self.f, "{float}"),
            LiteralKind::Str(string) => self.write_str(string),
            LiteralKind::True => self.write_str("true"),
            LiteralKind::False => self.write_str("false"),
        }
    }

    fn visit_expr_case(&mut self, expr_case: &super::ExprCase) -> fmt::Result {
        self.write_str("case ")?;
        self.visit_expr(&expr_case.expr)?;
        self.write_str(" do")?;

        if expr_case.arms.is_empty() {
            self.write_str(" ")?;
        } else {
            self.newline()?;
        }
        self.indent();

        for (pat, expr) in &expr_case.arms {
            self.visit_pat(pat)?;
            self.write_str(" -> ")?;
            self.visit_expr(expr)?;
            self.write_str(", ")?;
            self.newline()?;
        }

        self.dedent();
        self.write_str("end")
    }

    fn visit_expr_let(&mut self, expr_let: &super::ExprLet) -> fmt::Result {
        self.write_str("let ")?;
        self.visit_ident(&expr_let.lhs)?;

        let insert_newline = matches!(
            &*expr_let.rhs,
            Expr::Let(_) | Expr::If(_) | Expr::Case(_) | Expr::Fn(_)
        );

        if insert_newline {
            self.write_str(" =")?;

            self.newline()?;
            self.indent();
        } else {
            self.write_str(" = ")?;
            self.indent();
        }

        self.visit_expr(&expr_let.rhs)?;
        self.write_str(";")?;

        self.newline()?;
        self.dedent();

        self.visit_expr(&expr_let.body)?;

        Ok(())
    }

    fn visit_expr_unary(&mut self, expr_unary: &super::ExprUnary) -> fmt::Result {
        self.parenthesize(|this| super::visitor::visit_expr_unary(this, expr_unary))
    }

    fn visit_un_op(&mut self, un_op: &UnOp) -> fmt::Result {
        match un_op {
            UnOp::Neg => self.write_str("-"),
            UnOp::Not => self.write_str("!"),
        }
    }

    fn visit_expr_binary(&mut self, expr_binary: &super::ExprBinary) -> fmt::Result {
        self.parenthesize(|this| super::visitor::visit_expr_binary(this, expr_binary))
    }

    fn visit_bin_op(&mut self, bin_op: &BinOp) -> fmt::Result {
        match bin_op {
            BinOp::Eq => self.write_str(" == "),
            BinOp::Ne => self.write_str(" != "),
            BinOp::Lt => self.write_str(" < "),
            BinOp::Le => self.write_str(" <= "),
            BinOp::Gt => self.write_str(" > "),
            BinOp::Ge => self.write_str(" >= "),
            BinOp::Add => self.write_str(" + "),
            BinOp::Sub => self.write_str(" - "),
            BinOp::Mul => self.write_str(" * "),
            BinOp::Div => self.write_str(" / "),
            BinOp::And => self.write_str(" && "),
            BinOp::Or => self.write_str(" || "),
        }
    }

    fn visit_expr_if(&mut self, expr_if: &super::ExprIf) -> fmt::Result {
        self.write_str("if ")?;
        self.visit_expr(&expr_if.cond)?;

        self.write_str(" do")?;
        self.newline()?;
        self.indent();

        self.visit_expr(&expr_if.do_branch)?;

        if let Some(expr) = expr_if.else_branch.as_ref() {
            self.dedent();
            self.newline()?;
            self.write_str("else")?;

            self.newline()?;
            self.indent();
            self.visit_expr(expr)?;
        }

        self.newline()?;
        self.dedent();
        self.write_str("end")
    }

    fn visit_expr_fn(&mut self, expr_fn: &super::ExprFn) -> fmt::Result {
        self.write_str("fn ")?;

        for arg in &expr_fn.args {
            self.visit_ident(arg)?;
            self.write_str(" ")?;
        }

        self.write_str("->")?;
        self.newline()?;
        self.indent();

        self.visit_expr(&expr_fn.body)?;
        self.newline()?;

        self.dedent();
        self.write_str("end")
    }

    fn visit_expr_app(&mut self, expr_app: &super::ExprApp) -> fmt::Result {
        self.parenthesize(|this| {
            this.visit_expr(&expr_app.func)?;
            this.write_str(" ")?;
            this.visit_expr(&expr_app.arg)
        })
    }
}
