mod ast;
mod parser;
mod scanner;
mod token;

use std::{cell::RefCell, fmt::Display, io, path::Path};

use crate::{
    ast::pretty_printer::pretty_print,
    parser::Parser,
    scanner::Scanner,
    token::{Token, TokenKind},
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
        let mut parser = Parser::new(tokens, self);
        if let Ok(file) = parser.parse() {
            println!("{}", pretty_print(&file));
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
