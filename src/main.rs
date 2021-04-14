mod ast;
mod parser;

use std::env;
use std::fs;

use ast::Environment;
use parser::combinators::Parser;

fn main() {
    let source = fs::read_to_string(env::args().nth(1).unwrap()).unwrap();
    let parser = parser::make_full_parser();
    let ast = parser.parse(&source).unwrap().1;
    let mut output_asm = String::new();
    let mut env = Environment::default();
    ast.emit(&mut output_asm, &mut env);
    print!("{}", output_asm);
}
