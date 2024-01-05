#[derive(Debug, Clone)]
pub enum Expr {
    Eof,
    Lit(Lit),
    Prim0(Op0),
    Prim1(Op1, Box<Expr>),
    Prim2(Op2, Box<Expr>, Box<Expr>),
    Prim3(Op3, Box<Expr>, Box<Expr>, Box<Expr>),
    Begin(Box<Expr>, Box<Expr>),
    Variable(Identifier),
    Let(Let),
    App(App),
    If(If),
    Match(Match),
    Lambda(Lambda),
}

#[derive(Debug, Clone)]
pub struct Let {
    pub binding: Binding,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub lhs: Identifier,
    pub rhs: Box<Expr>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn new(s: &str) -> Identifier {
        Identifier(s.to_owned())
    }
}

/// Function application.
#[derive(Debug, Clone)]
pub struct App {
    pub function: Box<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct If {
    pub cond: Box<Expr>,
    pub then: Box<Expr>,
    pub els: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub expr: Box<Expr>,
    pub arms: Vec<Arm>,
}

#[derive(Debug, Clone)]
pub struct Arm {
    pub pattern: Pattern,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Variable(Identifier),
    Lit(Lit),
    Cons(Box<Pattern>, Box<Pattern>),
    Box(Box<Pattern>),
    And(Box<Pattern>, Box<Pattern>),
}

#[derive(Debug, Clone)]
pub enum Lit {
    Int(i64),
    Bool(bool),
    Char(char),
    String(String),
    EmptyList,
}

#[derive(Debug, Clone)]
pub enum Op0 {
    ReadByte,
    PeekByte,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Op3 {
    /// Sets the element of the vector at the given index to the given value.
    /// The first operand is the vector, the second operand is the index, and the third operand is the new value.
    VectorSet,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub function_definitions: Vec<FunctionDefinition>,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub signature: FunctionSignature,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: Identifier,
    pub params: Vec<Identifier>,
}

#[derive(Debug, Clone)]
pub struct Lambda {
    pub id: Identifier,
    pub params: Vec<Identifier>,
    pub body: Box<Expr>,
}

impl Into<Lambda> for FunctionDefinition {
    fn into(self) -> Lambda {
        Lambda {
            id: self.signature.name,
            params: self.signature.params,
            body: Box::new(self.body),
        }
    }
}
