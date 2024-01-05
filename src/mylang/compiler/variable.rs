use crate::{
    a86::ast::{Operand, Register, Statement},
    mylang::ast::{self, Identifier},
};

use super::{expr::compile_expr, state::Compiler};

const RAX: Operand = Operand::Register(Register::RAX);
const RSP: Operand = Operand::Register(Register::RSP);

pub fn compile_let(
    expr: ast::Let,
    compiler: &mut Compiler,
    env: &VariablesTable,
    is_tail_expr: bool,
) -> Vec<Statement> {
    let ast::Let { binding, body } = expr;

    let mut statements = compile_expr(*binding.rhs, compiler, env, false);
    statements.push(Statement::Push { src: RAX });

    let new_env = env.with_var(&binding.lhs);
    statements.extend(compile_expr(*body, compiler, &new_env, is_tail_expr));

    // Pop the value from the stack and discard it.
    statements.push(Statement::Add {
        dest: RSP,
        src: Operand::Immediate(8),
    });
    statements
}

pub fn compile_variable(
    variable: Identifier,
    _compiler: &mut Compiler,
    env: &VariablesTable,
) -> Vec<Statement> {
    let position = env
        .position(&variable)
        .expect(format!("Undefined variable `{}`", variable.0).as_str()); // TODO: Return `Result` type.
    let offset = (position * 8) as i64;
    vec![Statement::Mov {
        dest: RAX,
        src: Operand::Offset(Register::RSP, offset),
    }]
}

#[derive(Clone, Debug)]
pub struct VariablesTable {
    variables: Vec<Option<Identifier>>,
}

/// Keeps track of local variables, mapping them to lexical addresses.
impl VariablesTable {
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
        }
    }

    pub fn extended(&self, new_variables: impl IntoIterator<Item = Identifier>) -> Self {
        let mut variables = self.variables.clone();
        variables.extend(new_variables.into_iter().map(Some));
        Self::new_with_vars(&variables)
    }

    pub fn with_var(&self, variable: &Identifier) -> Self {
        let mut variables = self.variables.clone();
        variables.push(Some(variable.clone()));
        Self::new_with_vars(&variables)
    }

    pub fn with_non_var(&self) -> Self {
        let mut variables = self.variables.clone();
        variables.push(None);
        Self::new_with_vars(&variables)
    }

    pub fn position(&self, variable: &Identifier) -> Option<usize> {
        self.variables
            .iter()
            .position(|option| option.as_ref().is_some_and(|v| v == variable))
            .map(|i| self.variables.len() - i - 1)
    }
    
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    fn new_with_vars(variables: &Vec<Option<Identifier>>) -> Self {
        Self {
            variables: variables.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_position() {
        let a = Identifier("a".to_string());
        let b = Identifier("b".to_string());
        let c = Identifier("c".to_string());

        let env = VariablesTable::new().with_var(&a).with_var(&b).with_var(&c);

        assert_eq!(env.position(&a), Some(2));
        assert_eq!(env.position(&b), Some(1));
        assert_eq!(env.position(&c), Some(0));
    }
}
