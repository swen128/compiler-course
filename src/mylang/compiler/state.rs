use std::collections::{HashMap, HashSet};

pub struct Compiler {
    last_label_id: usize,
    string_literals: HashMap<String, Label>,
}

impl Compiler {
    pub fn new(string_literals: HashSet<String>) -> Compiler {
        Compiler {
            last_label_id: string_literals.len(),
            string_literals: string_literals
                .into_iter()
                .enumerate()
                .map(|(i, string)| (string, Label(format!("string_data_{}", i))))
                .collect(),
        }
    }

    pub fn new_label_id(&mut self) -> String {
        self.last_label_id += 1;
        self.last_label_id.to_string()
    }
    
    pub fn string_literal_label(&self, string: &str) -> Option<&Label> {
        self.string_literals.get(string)
    }
    
    pub fn string_literals(&self) -> &HashMap<String, Label> {
        &self.string_literals
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Label(pub String);
