use crate::a86::ast::*;

pub struct CompilationContext {
    pub platform: Platform,
}

pub enum Platform {
    Linux,
    MacOS,
}


pub fn print(program: &Program, context: &CompilationContext) -> String {
    let mut output = String::new();
    for statement in &program.statements {
        output.push_str(&print_statement(&statement, &context));
        output.push_str("\n");
    }
    output
}

fn print_statement(statement: &Statement, context: &CompilationContext) -> String {
    match statement {
        Statement::Global { name } => format!("global {}", name),
        Statement::Label { name } => format!("{}:", name),
        Statement::Mov { dest, src } => print_mov(dest, src),
        Statement::Cmp { dest, src } => print_cmp(dest, src),
        Statement::Je { label } => format!("je {}", label),
        Statement::Push { src } => format!("push {}", print_operand(src)),
        Statement::Pop { dest } => format!("pop {}", print_operand(dest)),
        Statement::Add { dest, src } => print_add(dest, src),
        Statement::Sub { dest, src } => print_sub(dest, src),
        Statement::Call { label } => print_label(label, &context),
        Statement::Ret => "ret".to_string(),
    }
}

fn print_mov(dest: &Operand, src: &Operand) -> String {
    format!("mov {}, {}", print_operand(dest), print_operand(src))
}

fn print_cmp(dest: &Operand, src: &Operand) -> String {
    format!("cmp {}, {}", print_operand(dest), print_operand(src))
}

fn print_add(dest: &Operand, src: &Operand) -> String {
    format!("add {}, {}", print_operand(dest), print_operand(src))
}

fn print_sub(dest: &Operand, src: &Operand) -> String {
    format!("sub {}, {}", print_operand(dest), print_operand(src))
}

fn print_label(label: &String, context: &CompilationContext) -> String {
    match context.platform {
        Platform::Linux => format!("_{}", label),
        Platform::MacOS => label.to_string(),
    }
}

fn print_operand(operand: &Operand) -> String {
    match operand {
        Operand::Memory(name) => {
            format!("[{}]", name)
        }
        Operand::Immediate(value) => {
            format!("{}", value)
        }
        Operand::Register(register) => print_register(register),
    }
}

fn print_register(register: &Register) -> String {
    match register {
        Register::RAX => "rax".to_string(),
        Register::RBX => "rbx".to_string(),
    }
}
