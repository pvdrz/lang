pub(crate) enum MonoTy {
    Var(MonoVar),
    Int,
    Float,
    String,
    Bool,
    Fn(FnMonoTy),
}

pub(crate) struct FnMonoTy {
    pub(crate) arg: Box<MonoTy>,
    pub(crate) ret: Box<MonoTy>,
}

pub(crate) struct MonoVar {
    inner: usize,
}

pub(crate) struct MonoVarGen {
    count: usize,
}

impl MonoVarGen {
    pub(crate) fn generate(&mut self) -> MonoTy {
        let var = MonoVar { inner: self.count };
        self.count += 1;
        MonoTy::Var(var)
    }

    pub(crate) fn new() -> Self {
        Self { count: 0 }
    }
}
