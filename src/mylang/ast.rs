#[derive(Debug)]
pub enum Expr {
    Eof,
    Lit(Lit),
    Prim0(Op0),
    Prim1(Op1, Box<Expr>),
    Begin(Box<Expr>, Box<Expr>),
    Variable(Variable),
    Let(Let),
    If(If),
}

#[derive(Debug)]
pub struct Let {
    pub binding: Binding,
    pub body: Box<Expr>,
}

#[derive(Debug)]
pub struct Binding {
    pub lhs: Variable,
    pub rhs: Box<Expr>,
}

#[derive(Debug, PartialEq)]

pub struct Variable(pub String);

impl Variable {
    pub fn new(s: &str) -> Variable {
        Variable(s.to_owned())
    }
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
pub enum Op0 {
    ReadByte,
    PeekByte,
}

#[derive(Debug)]
pub enum Op1 {
    Add1,
    Sub1,
    IsZero,
    IsChar,
    IsEof,
    IntToChar,
    CharToInt,
    WriteByte,
}

#[derive(Debug)]
pub enum Op2 {
    Begin,
}

#[derive(Debug)]
pub struct Program {
    pub expr: Expr,
}

#[derive(Debug)]
pub enum Operator {
    Op1(Op1),
}
