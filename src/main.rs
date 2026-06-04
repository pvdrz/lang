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
mod source_map;

use std::{cell::RefCell, fmt::Display, io, ops::DerefMut, path::Path};

use crate::{
    ast::pretty_printer::pretty_print,
    ir::{DefId, DefIdGen},
    lowering::Resolver,
    parser::Parser,
    scanner::Scanner,
    source_map::{SourceMap, Span},
    ty::{Ty, VarTyGen},
    tycheck::TyChecker,
};

struct Lang {
    had_error: RefCell<bool>,
    source_map: RefCell<SourceMap>,
    var_ty_gen: RefCell<VarTyGen>,
    def_id_gen: RefCell<DefIdGen>,
}

impl Lang {
    fn new() -> Self {
        Self {
            had_error: false.into(),
            source_map: SourceMap::new().into(),
            var_ty_gen: VarTyGen::new().into(),
            def_id_gen: DefIdGen::new().into(),
        }
    }

    fn error(&self, span: Span, msg: impl Display) {
        *self.had_error.borrow_mut() = true;
        let (line, col) = self.source_map().map_offset(span.start());
        eprintln!("Error at {}:{}: {msg}", line + 1, col + 1);
    }

    fn gen_var_ty(&self) -> Ty {
        let var = self.var_ty_gen.borrow_mut().generate();
        Ty::Var(var)
    }

    fn gen_def_id(&self) -> DefId {
        self.def_id_gen.borrow_mut().generate()
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

        let mut resolver = Resolver::new(self);
        let mut file = resolver.lower_expr(&file);
        if *self.had_error.borrow() {
            return;
        }
        let mut checker = TyChecker::new(self);
        let file_ty = checker.infer_type(&mut file);
        if *self.had_error.borrow() {
            return;
        }

        println!("Program has type: {file_ty}");
    }

    fn source_map(&self) -> impl DerefMut<Target = SourceMap> {
        self.source_map.borrow_mut()
    }
}

fn main() {
    let lang = Lang::new();
    let args: Vec<_> = std::env::args_os().collect();
    lang.run_file(&args[1]).unwrap();
}
