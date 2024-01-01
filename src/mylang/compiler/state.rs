use crate::mylang::ast::Identifier;

pub struct Compiler {
    last_label_id: usize,
    pub variables_table: VariablesTable,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            last_label_id: 0,
            variables_table: VariablesTable::new(),
        }
    }

    pub fn new_label_id(&mut self) -> String {
        self.last_label_id += 1;
        self.last_label_id.to_string()
    }
}

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

    /// Pushes a new variable to the stack.
    /// This should be called when binding a new variable in the `let` expression.
    pub fn push_variable(&mut self, variable: Identifier) {
        self.variables.push(Some(variable));
    }

    /// Pushes a new non-variable to the stack.
    /// This should be called whenever pushing an arbitrary non-variable value to the stack.
    pub fn push_non_variable(&mut self) {
        self.variables.push(None);
    }

    pub fn pop(&mut self) {
        self.variables.pop();
    }

    pub fn position(&self, variable: &Identifier) -> Option<usize> {
        self.variables
            .iter()
            .position(|option| option.as_ref().is_some_and(|v| v == variable))
            .map(|i| self.variables.len() - i - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_position() {
        let mut variables_table = VariablesTable::new();
        variables_table.push_variable(Identifier("a".to_string()));
        variables_table.push_variable(Identifier("b".to_string()));
        variables_table.push_variable(Identifier("c".to_string()));
        variables_table.pop();
        assert_eq!(
            variables_table.position(&Identifier("a".to_string())),
            Some(1)
        );
        assert_eq!(
            variables_table.position(&Identifier("b".to_string())),
            Some(0)
        );
        assert_eq!(variables_table.position(&Identifier("c".to_string())), None);
    }
}
