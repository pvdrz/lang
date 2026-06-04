pub(crate) mod mono;

pub(crate) enum Ty {
    Int,
    Float,
    String,
    Bool,
    Fn(FnTy),
}

pub(crate) struct FnTy {
    pub(crate) arg: Box<Ty>,
    pub(crate) ret: Box<Ty>,
}
