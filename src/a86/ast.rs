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
}

pub struct Program {
    pub statements: Vec<Statement>,
}
