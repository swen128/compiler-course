pub struct Compiler {
    last_label_id: usize,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler { last_label_id: 0 }
    }

    pub fn new_label_id(&mut self) -> String {
        self.last_label_id += 1;
        self.last_label_id.to_string()
    }
}
