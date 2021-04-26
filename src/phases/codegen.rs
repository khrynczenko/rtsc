use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Environment {
    pub locals: HashMap<String, isize>,
    pub next_local_offset: isize,
}

impl Environment {
    pub fn new(locals: HashMap<String, isize>, next_local_offset: isize) -> Self {
        Environment {
            locals,
            next_local_offset,
        }
    }
}

pub trait CodeGenerator {
    fn emit(&self, buffer: &mut String, env: &mut Environment);
}
