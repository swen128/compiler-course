pub enum Expr {
    Lit(Lit),
    Prim1(Op1, Box<Expr>),
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
