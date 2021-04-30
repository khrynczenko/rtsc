mod ast;
mod parser;
mod phases;
mod types;

use std::env;
use std::fs;

use linked_hash_map::LinkedHashMap;

use ast::Ast;
use parser::combinators::Parser;
use phases::codegen::{Arm32Generator, CodeGenerator, Environment};
use phases::typecheck::{StaticTypeChecker, TypeChecker};
use types::Type;

fn parse(source: &str) -> Ast {
    let parser = parser::make_full_parser();
    parser.parse(&source).unwrap().1
}

fn typecheck(ast: &Ast) {
    let mut functions = LinkedHashMap::new();
    // I kind of use `putchar` function in test so I add it to the available funcions at start
    let mut putchar_parameters = LinkedHashMap::new();
    putchar_parameters.insert(String::from("x1"), Type::Number);
    functions.insert(
        String::from("putchar"),
        Type::Function {
            parameter_types: putchar_parameters,
            return_type: Box::new(Type::Void),
        },
    );
    StaticTypeChecker::new(LinkedHashMap::new(), functions, None).check(&ast);
}

fn generate_code(ast: Ast) -> String {
    let mut output_asm = String::new();
    let mut env = Environment::default();
    Arm32Generator::new(ast).emit(&mut output_asm, &mut env);
    output_asm
}

fn compile(source: &str) -> String {
    let ast = parse(source);
    typecheck(&ast);
    generate_code(ast)
}

fn main() {
    let source = fs::read_to_string(env::args().nth(1).unwrap()).unwrap();
    print!("{}", compile(&source));
}
