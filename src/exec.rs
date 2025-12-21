// exec

use crate::ast::Program;

pub struct Executor {
    name: String,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            name: "Exec not unimplemented".to_string(),
        }
    }

    pub fn exec(&self, ast: &Program) -> u64 {
        println!("{}", self.name.as_str());
        return 0;
    }
}
