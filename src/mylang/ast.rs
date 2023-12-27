#[derive(Debug)]
pub enum Expr {
    Lit(Lit),
    Prim1(Op1, Box<Expr>),
    If(If),
}

#[derive(Debug)]
pub struct If {
    pub cond: Box<Expr>,
    pub then: Box<Expr>,
    pub els: Box<Expr>,
}

#[derive(Debug)]
pub enum Lit {
    Int(i64),
    Bool(bool),
    Char(char),
}

#[derive(Debug)]
pub enum Op1 {
    Add1,
    Sub1,
    IsZero,
    IsChar,
    IntToChar,
    CharToInt,
}

#[derive(Debug)]
pub struct Program {
    pub expr: Expr,
}

#[derive(Debug)]
pub enum Operator {
    Op1(Op1),
}
