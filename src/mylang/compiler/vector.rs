use crate::{
    a86::ast::{Operand, Register, Statement},
    mylang::data_type::{Value, INT_TYPE, VECTOR_TYPE},
};

use super::{
    error::ERR_LABEL,
    state::Compiler,
    types::{assert_natural_number, assert_vector},
};

const RAX: Operand = Operand::Register(Register::RAX);
const RBX: Operand = Operand::Register(Register::RBX);
const R8: Operand = Operand::Register(Register::R8);
const R9: Operand = Operand::Register(Register::R9);
const R10: Operand = Operand::Register(Register::R10);

/// Returns instructions to initialize a vector of the given length with the repeated values,
/// assuming the length and the value is already given in r8 and rax respectively.
pub fn compile_make_vector(compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = assert_natural_number(Register::R8);

    let loop_label = format!("loop_{}", compiler.new_label_id());
    let end_label = format!("end_{}", compiler.new_label_id());
    let empty_label = format!("empty_{}", compiler.new_label_id());

    // Special case for empty vector
    statements.push(Statement::Cmp {
        dest: R8,
        src: Operand::from(Value::Int(0)),
    });
    statements.push(Statement::Je {
        label: empty_label.clone(),
    });

    // Stash the top address and cast it to the vector type.
    statements.push(Statement::Mov { dest: R9, src: RBX });
    statements.push(Statement::Or {
        dest: R9,
        src: Operand::Immediate(VECTOR_TYPE.tag.0 as i64),
    });

    // Put the length of the vector at the beginning.
    // Note: The length is guaranteed to be an integer, so we can strip the type tag.
    statements.push(Statement::Sar {
        dest: R8,
        src: Operand::Immediate(INT_TYPE.shift as i64),
    });
    statements.push(Statement::Mov {
        dest: Operand::Offset(Register::RBX, 0),
        src: R8,
    });
    statements.push(Statement::Add {
        dest: RBX,
        src: Operand::Immediate(8),
    });

    // Initialize each element of the vector to the given value.
    statements.push(Statement::Label {
        name: loop_label.clone(),
    });
    statements.push(Statement::Mov {
        dest: Operand::Offset(Register::RBX, 0),
        src: RAX,
    });
    statements.push(Statement::Add {
        dest: RBX,
        src: Operand::Immediate(8),
    });
    statements.push(Statement::Sub {
        dest: R8,
        src: Operand::Immediate(1),
    });
    statements.push(Statement::Cmp {
        dest: R8,
        src: Operand::Immediate(0),
    });
    statements.push(Statement::Jne {
        label: loop_label.clone(),
    });

    // Return the vector.
    statements.push(Statement::Mov { dest: RAX, src: R9 });
    statements.push(Statement::Jmp {
        label: end_label.clone(),
    });

    // Special case for empty vector
    statements.push(Statement::Label {
        name: empty_label.clone(),
    });
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::from(Value::EmptyVector),
    });

    statements.push(Statement::Label {
        name: end_label.clone(),
    });
    statements
}

/// Returns instructions which sets rax to the element in the vector at the given index,
/// assuming the vector and the index is already given in r8 and rax respectively.
pub fn compile_vector_ref(_compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = assert_vector(Register::R8);
    statements.extend(assert_natural_number(Register::RAX));

    // Special case for empty vector
    statements.push(Statement::Cmp {
        dest: R8,
        src: Operand::from(Value::EmptyVector),
    });
    statements.push(Statement::Je {
        label: ERR_LABEL.to_string(),
    });

    // Cast r8 to the raw pointer address of the vector.
    statements.push(Statement::Xor {
        dest: R8,
        src: Operand::Immediate(VECTOR_TYPE.tag.0 as i64),
    });
    // Set r9 to the length of the vector.
    statements.push(Statement::Mov {
        dest: R9,
        src: Operand::Offset(Register::R8, 0),
    });
    // Cast rax to raw integer representing the index.
    statements.push(Statement::Sar {
        dest: RAX,
        src: Operand::Immediate(INT_TYPE.shift as i64),
    });

    // Check if the index is out of bounds. (length - 1 < index)
    statements.push(Statement::Sub {
        dest: R9,
        src: Operand::Immediate(1),
    });
    statements.push(Statement::Cmp { dest: R9, src: RAX });
    statements.push(Statement::Jl {
        label: ERR_LABEL.to_string(),
    });

    // Get the element at the given index.
    statements.push(Statement::Sal {
        dest: RAX,
        src: Operand::Immediate(3), // Each element takes up 2^3 bytes.
    });
    statements.push(Statement::Add { dest: R8, src: RAX });
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::Offset(Register::R8, 8), // The offset 8 is required as the first word is the length.
    });

    statements
}

/// Returns instructions which mutates the element in the given vector at the given index,
/// which are assumed to be laid out as follows:
/// * 1st argument (vector): Second of the stack
/// * 2nd argument (index): Topmost of the stack
/// * 3rd argument (new value): rax
/// 
/// This clobbers r8, r9, and r10.
pub fn compile_vector_set(compiler: &mut Compiler) -> Vec<Statement> {
    // Move the arguments to the registers.
    // 1st argument (vector): r8
    // 2nd argument (index): r10
    // 3rd argument (new value): rax
    let mut statements = vec![
        Statement::Pop {
            dest: Operand::Register(Register::R10),
        },
        Statement::Pop {
            dest: Operand::Register(Register::R8),
        },
    ];
    compiler.variables_table.pop();
    compiler.variables_table.pop();
    statements.extend(assert_vector(Register::R8));
    statements.extend(assert_natural_number(Register::R10));

    // Cast r8 to the raw pointer address of the vector.
    statements.push(Statement::Xor {
        dest: R8,
        src: Operand::Immediate(VECTOR_TYPE.tag.0 as i64),
    });
    // Set r9 to the length of the vector.
    statements.push(Statement::Mov {
        dest: R9,
        src: Operand::Offset(Register::R8, 0),
    });
    // Cast r10 to raw integer representing the index.
    statements.push(Statement::Sar {
        dest: R10,
        src: Operand::Immediate(INT_TYPE.shift as i64),
    });

    // Check if the index is out of bounds. (length - 1 < index)
    statements.push(Statement::Sub {
        dest: R9,
        src: Operand::Immediate(1),
    });
    statements.push(Statement::Cmp { dest: R9, src: R10 });
    statements.push(Statement::Jl {
        label: ERR_LABEL.to_string(),
    });

    // Set the new value.
    statements.push(Statement::Sal {
        dest: R10,
        src: Operand::Immediate(3), // Multiply by 2^3 = 8 = 1 word
    });
    statements.push(Statement::Add { dest: R8, src: R10 });
    statements.push(Statement::Mov {
        dest: Operand::Offset(Register::R8, 8), // The offset 8 is required as the first word is the length.
        src: RAX,
    });

    // Return void.
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::from(Value::Void),
    });
    statements
}
