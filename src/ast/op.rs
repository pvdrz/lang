#[derive(Debug)]
pub(crate) enum UnOp {
    Neg,
    Not,
}

#[derive(Debug)]
pub(crate) enum BinOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
}
