mod ast;
mod parser;
mod phases;

use std::env;
use std::fs;

use parser::combinators::Parser;
use phases::codegen::{CodeGenerator, Environment};

fn main() {
    let source = fs::read_to_string(env::args().nth(1).unwrap()).unwrap();
    let parser = parser::make_full_parser();
    let ast = parser.parse(&source).unwrap().1;
    let mut output_asm = String::new();
    let mut env = Environment::default();
    ast.emit(&mut output_asm, &mut env);
    print!("{}", output_asm);
}
