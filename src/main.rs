mod ast;
mod ir;
mod lowering;
mod parser;
mod scanner;
mod token;
mod ty;
mod tycheck;
#[macro_use]
mod utils;

use std::{cell::RefCell, fmt::Display, io, path::Path};

use crate::{
    ast::pretty_printer::pretty_print,
    lowering::Resolver,
    parser::Parser,
    scanner::Scanner,
    token::{Token, TokenKind},
    ty::mono::MonoVarGen,
};

struct Lang {
    had_error: RefCell<bool>,
}

impl Lang {
    fn new() -> Self {
        Self {
            had_error: false.into(),
        }
    }

    fn error(&self, line: usize, msg: impl Display) {
        *self.had_error.borrow_mut() = true;
        self.report(line, "", msg)
    }

    fn report(&self, line: usize, where_: impl Display, msg: impl Display) {
        eprintln!("[line.{line}] error{where_}: {msg}");
    }

    fn run_file<P: AsRef<Path>>(&self, path: &P) -> io::Result<()> {
        let string = std::fs::read_to_string(path)?;
        self.run(string);

        Ok(())
    }

    fn run(&self, source: String) {
        let scanner = Scanner::new(source, self);
        let tokens = scanner.scan_tokens();
        if *self.had_error.borrow() {
            return;
        }

        let mut parser = Parser::new(tokens, self);
        let Ok(file) = parser.parse() else {
            return;
        };
        println!("{}", pretty_print(&file));

        let mut mono_var_gen = MonoVarGen::new();
        let mut resolver = Resolver::new(self, &mut mono_var_gen);
        let _file = resolver.lower_expr(&file);
        if *self.had_error.borrow() {
            return;
        }
    }

    fn parse_error(&self, token: &Token, msg: impl Display) {
        if token.kind == TokenKind::EOF {
            self.report(token.line, " at end", msg)
        } else {
            self.report(token.line, "", msg)
        }
    }
}

fn main() {
    let lang = Lang::new();
    let args: Vec<_> = std::env::args_os().collect();
    lang.run_file(&args[1]).unwrap();
}
