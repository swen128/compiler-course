#[derive(Debug)]
pub enum Expr {
    Eof,
    Lit(Lit),
    String(String),
    Prim0(Op0),
    Prim1(Op1, Box<Expr>),
    Prim2(Op2, Box<Expr>, Box<Expr>),
    Prim3(Op3, Box<Expr>, Box<Expr>, Box<Expr>),
    Begin(Box<Expr>, Box<Expr>),
    Variable(Identifier),
    Let(Let),
    App(App),
    If(If),
}

#[derive(Debug)]
pub struct Let {
    pub binding: Binding,
    pub body: Box<Expr>,
}

#[derive(Debug)]
pub struct Binding {
    pub lhs: Identifier,
    pub rhs: Box<Expr>,
}

#[derive(Debug, PartialEq)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn new(s: &str) -> Identifier {
        Identifier(s.to_owned())
    }
}

/// Function application.
#[derive(Debug)]
pub struct App {
    pub function: Identifier,
    pub args: Vec<Expr>,
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
    EmptyList,
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
    IsBox,
    IsCons,
    IsVector,
    IsString,
    IntToChar,
    CharToInt,
    WriteByte,
    Box,
    Unbox,
    Car,
    Cdr,
}

#[derive(Debug)]
pub enum Op2 {
    Add,
    Sub,
    /// Returns true if the first operand is less than the second operand. 
    LessThan,
    Equal,
    Cons,
    /// Returns a new vector of the given length, with each element initialized to the given value.
    /// The first operand is the length of the vector, and the second operand is the initial value.
    MakeVector,
    /// Returns a new string of the given length, with each character initialized to the given value.
    /// The first operand is the length of the string, and the second operand is the character.
    MakeString,
    /// Returns the element of the vector at the given index.
    /// The first operand is the vector, and the second operand is the index.
    VectorRef,
    /// Returns the character in the string at the given index.
    /// The first operand is the string, and the second operand is the index.
    StringRef,
}

#[derive(Debug)]
pub enum Op3 {
    /// Sets the element of the vector at the given index to the given value.
    /// The first operand is the vector, the second operand is the index, and the third operand is the new value.
    VectorSet,
}

#[derive(Debug)]
pub struct Program {
    pub function_definitions: Vec<FunctionDefinition>,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct FunctionDefinition {
    pub signature: FunctionSignature,
    pub body: Expr,
}

#[derive(Debug)]
pub struct FunctionSignature {
    pub name: Identifier,
    pub params: Vec<Identifier>,
}
