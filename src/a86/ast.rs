#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Global { name: String },
    Extern { name: String },
    Label { name: String },
    Mov { dest: Operand, src: Operand },
    And { dest: Operand, src: Operand },
    Xor { dest: Operand, src: Operand },
    Sar { dest: Operand, src: Operand },
    Sal { dest: Operand, src: Operand },
    Cmp { dest: Operand, src: Operand },
    Cmove { dest: Operand, src: Operand },
    Jmp { label: String },
    Je { label: String },
    Jne { label: String },
    Jg { label: String },
    Jl { label: String },
    Push { src: Operand },
    Pop { dest: Operand },
    Add { dest: Operand, src: Operand },
    Sub { dest: Operand, src: Operand },
    Call { label: String },
    Ret,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operand {
    Memory(String),
    Immediate(i64),
    Register(Register),
    Offset(Register, i64),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Register {
    RAX,
    RBX,
    RDI,
    RSP,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

pub struct Program {
    pub statements: Vec<Statement>,
}
