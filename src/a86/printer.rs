use super::ast::*;

pub struct CompilationContext {
    pub platform: Platform,
}

#[allow(dead_code)]
pub enum Platform {
    Linux,
    MacOS,
}

pub fn print(program: &Program, context: &CompilationContext) -> String {
    let mut output = String::new();

    output.push_str("\tdefault rel\n");
    output.push_str("\tsection .text\n");

    for statement in &program.statements {
        output.push_str(&print_statement(&statement, &context));
        output.push_str("\n");
    }
    output
}

fn print_statement(statement: &Statement, context: &CompilationContext) -> String {
    match statement {
        Statement::Global { name } => format!("\tglobal {}", name),
        Statement::Extern { name } => format!("\textern {}", name),
        Statement::Label { name } => format!("{}:", print_label(name, context)),
        Statement::Mov { dest, src } => print_mov(dest, src),
        Statement::And { dest, src } => print_and(dest, src),
        Statement::Or { dest, src } => print_or(dest, src),
        Statement::Xor { dest, src } => print_xor(dest, src),
        Statement::Sar { dest, src } => print_sar(dest, src),
        Statement::Sal { dest, src } => print_sal(dest, src),
        Statement::Cmp { dest, src } => print_cmp(dest, src),
        Statement::Cmove { dest, src } => print_cmove(dest, src),
        Statement::Cmovl { dest, src } => print_cmovl(dest, src),
        Statement::Je { label } => format!("\tje {}", print_label(label, context)),
        Statement::Jne { label } => format!("\tjne {}", print_label(label, context)),
        Statement::Jg { label } => format!("\tjg {}", print_label(label, context)),
        Statement::Jl { label } => format!("\tjl {}", print_label(label, context)),
        Statement::Jmp { label } => format!("\tjmp {}", print_label(label, context)),
        Statement::JmpRegister(register) => format!("\tjmp {}", print_register(register)),
        Statement::Push { src } => format!("\tpush {}", print_operand(src)),
        Statement::Pop { dest } => format!("\tpop {}", print_operand(dest)),
        Statement::Add { dest, src } => print_add(dest, src),
        Statement::Sub { dest, src } => print_sub(dest, src),
        Statement::Lea { dest, label } => print_lea(dest, label, context),
        Statement::Call { label } => print_call(label, context),
        Statement::Ret => "\tret".to_string(),
    }
}

fn print_mov(dest: &Operand, src: &Operand) -> String {
    format!("\tmov {}, {}", print_operand(dest), print_operand(src))
}

fn print_and(dest: &Operand, src: &Operand) -> String {
    format!("\tand {}, {}", print_operand(dest), print_operand(src))
}

fn print_or(dest: &Operand, src: &Operand) -> String {
    format!("\tor {}, {}", print_operand(dest), print_operand(src))
}

fn print_xor(dest: &Operand, src: &Operand) -> String {
    format!("\txor {}, {}", print_operand(dest), print_operand(src))
}

fn print_sar(dest: &Operand, src: &Operand) -> String {
    format!("\tsar {}, {}", print_operand(dest), print_operand(src))
}

fn print_sal(dest: &Operand, src: &Operand) -> String {
    format!("\tsal {}, {}", print_operand(dest), print_operand(src))
}

fn print_cmp(dest: &Operand, src: &Operand) -> String {
    format!("\tcmp {}, {}", print_operand(dest), print_operand(src))
}

fn print_cmove(dest: &Operand, src: &Operand) -> String {
    format!("\tcmove {}, {}", print_operand(dest), print_operand(src))
}

fn print_cmovl(dest: &Operand, src: &Operand) -> String {
    format!("\tcmovl {}, {}", print_operand(dest), print_operand(src))
}

fn print_add(dest: &Operand, src: &Operand) -> String {
    format!("\tadd {}, {}", print_operand(dest), print_operand(src))
}

fn print_sub(dest: &Operand, src: &Operand) -> String {
    format!("\tsub {}, {}", print_operand(dest), print_operand(src))
}

fn print_lea(dest: &Operand, label: &String, context: &CompilationContext) -> String {
    format!(
        "\tlea {}, {}",
        print_operand(dest),
        print_label(label, &context)
    )
}

fn print_call(label: &String, context: &CompilationContext) -> String {
    format!("\tcall {}", print_label(label, &context))
}

fn print_label(label: &String, context: &CompilationContext) -> String {
    match context.platform {
        Platform::MacOS => format!("_{}", label),
        Platform::Linux => label.to_string(),
    }
}

fn print_operand(operand: &Operand) -> String {
    match operand {
        Operand::Immediate(value) => {
            format!("{}", value)
        }
        Operand::Register(register) => print_register(register),

        Operand::Offset(register, offset) => {
            if offset >= &0 {
                format!("[{} + {}]", print_register(register), offset)
            } else {
                format!("[{} - {}]", print_register(register), offset.abs())
            }
        }
    }
}

fn print_register(register: &Register) -> String {
    match register {
        Register::RAX => "rax".to_string(),
        Register::EAX => "eax".to_string(),
        Register::RBX => "rbx".to_string(),
        Register::RDI => "rdi".to_string(),
        Register::RSP => "rsp".to_string(),
        Register::R1 => "r1".to_string(),
        Register::R2 => "r2".to_string(),
        Register::R3 => "r3".to_string(),
        Register::R4 => "r4".to_string(),
        Register::R5 => "r5".to_string(),
        Register::R6 => "r6".to_string(),
        Register::R7 => "r7".to_string(),
        Register::R8 => "r8".to_string(),
        Register::R9 => "r9".to_string(),
        Register::R10 => "r10".to_string(),
        Register::R11 => "r11".to_string(),
        Register::R12 => "r12".to_string(),
        Register::R13 => "r13".to_string(),
        Register::R14 => "r14".to_string(),
        Register::R15 => "r15".to_string(),
        Register::R9D => "r9d".to_string(),
    }
}
