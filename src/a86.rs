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

pub fn print(program: Program) -> String {
    let mut output = String::new();
    for statement in program.statements {
        output.push_str(&print_statement(statement));
        output.push_str("\n");
    }
    output
}

fn print_statement(statement: Statement) -> String {
    match statement {
        Statement::Global { name } => {
            format!("global {}", name)
        }
        Statement::Label { name } => {
            format!("{}:", name)
        }
        Statement::Mov { dest, src } => {
            format!("mov {}, {}", print_operand(dest), print_operand(src))
        }
        Statement::Cmp { dest, src } => {
            format!("cmp {}, {}", print_operand(dest), print_operand(src))
        }
        Statement::Je { label } => {
            format!("je {}", label)
        }
        Statement::Push { src } => {
            format!("push {}", print_operand(src))
        }
        Statement::Pop { dest } => {
            format!("pop {}", print_operand(dest))
        }
        Statement::Add { dest, src } => {
            format!("add {}, {}", print_operand(dest), print_operand(src))
        }
        Statement::Sub { dest, src } => {
            format!("sub {}, {}", print_operand(dest), print_operand(src))
        }
        Statement::Call { label } => {
            format!("call {}", label)
        }
        Statement::Ret => {
            "ret".to_string()
        }
    }
}

fn print_operand(operand: Operand) -> String {
    match operand {
        Operand::Memory(name) => {
            format!("[{}]", name)
        }
        Operand::Immediate(value) => {
            format!("{}", value)
        }
        Operand::Register(register) => {
            print_register(register)
        }
    }
}

fn print_register(register: Register) -> String {
    match register {
        Register::RAX => {
            "rax".to_string()
        }
        Register::RBX => {
            "rbx".to_string()
        }
    }
}
