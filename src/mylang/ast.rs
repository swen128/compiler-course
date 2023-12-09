pub enum Expr {
    Lit(Lit),
    Prim1(Op1, Box<Expr>),
    IfZero(IfZero),
}

pub struct IfZero {
    pub cond: Box<Expr>,
    pub then: Box<Expr>,
    pub els: Box<Expr>,
}

pub enum Lit {
    Int(i64),
}

pub enum Op1 {
    Add1,
    Sub1,
}

pub struct Program {
    pub expr: Expr,
}

pub enum Operator {
    Op1(Op1),
    IfZero,
}
