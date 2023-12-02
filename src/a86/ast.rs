pub enum Statement {
    Global { name: String },
    Label { name: String },
    Mov { dest: Operand, src: Operand },
    Cmp { dest: Operand, src: Operand },
    Je { label: String },
    Push { src: Operand },
    Pop { dest: Operand },
    Add { dest: Operand, src: Operand },
    Sub { dest: Operand, src: Operand },
    Call { label: String },
    Ret,
}

pub enum Operand {
    Memory(String),
    Immediate(i64),
    Register(Register),
}

pub enum Register {
    RAX,
    RBX,
}

pub struct Program {
    pub statements: Vec<Statement>,
}
