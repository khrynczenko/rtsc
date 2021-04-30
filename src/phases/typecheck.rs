use crate::types::Type;
use linked_hash_map::LinkedHashMap;

use crate::ast::Ast;

pub trait TypeChecker {
    fn check(&mut self, ast: &Ast) -> Type;
}

#[derive(Debug)]
pub struct StaticTypeChecker {
    locals: LinkedHashMap<String, Type>,
    functions: LinkedHashMap<String, Type>,
    current_return_type: Option<Type>,
}

impl StaticTypeChecker {
    pub fn new(
        locals: LinkedHashMap<String, Type>,
        functions: LinkedHashMap<String, Type>,
        current_return_type: Option<Type>,
    ) -> StaticTypeChecker {
        StaticTypeChecker {
            locals,
            functions,
            current_return_type,
        }
    }

    fn assert_type(lhs: Type, rhs: Type) {
        if lhs == Type::Undefined || rhs == Type::Undefined {
            return;
        }
        if lhs != rhs {
            panic!("Type mismatch {:?} != {:?}", lhs, rhs);
        }
    }
}

impl TypeChecker for StaticTypeChecker {
    fn check(&mut self, ast: &Ast) -> Type {
        match ast {
            Ast::Number(_) => Type::Number,
            Ast::Bool(_) => Type::Boolean,
            Ast::Undefined => Type::Undefined,
            Ast::Null => Type::Void,
            Ast::Not(expr) => {
                StaticTypeChecker::assert_type(Type::Boolean, self.check(expr));
                Type::Boolean
            }
            Ast::Addition(lhs, rhs)
            | Ast::Subtraction(lhs, rhs)
            | Ast::Multiplication(lhs, rhs)
            | Ast::Division(lhs, rhs) => {
                StaticTypeChecker::assert_type(Type::Number, self.check(lhs));
                StaticTypeChecker::assert_type(Type::Number, self.check(rhs));
                Type::Number
            }
            Ast::Equal(lhs, rhs) | Ast::NotEqual(lhs, rhs) => {
                StaticTypeChecker::assert_type(self.check(lhs), self.check(rhs));
                Type::Boolean
            }
            Ast::Var(name, expr) => {
                let t = self.check(expr);
                self.locals.insert(name.clone(), t);
                Type::Void
            }
            Ast::Identifier(name) => {
                if let Some(t) = self.locals.get(name) {
                    t.clone()
                } else {
                    panic!("Undefined variable {}", name);
                }
            }
            Ast::Assignment(name, expr) => {
                if let Some(t) = self.locals.get(name) {
                    StaticTypeChecker::assert_type(t.clone(), self.check(expr));
                } else {
                    panic!("Undefined variable {}", name);
                }
                Type::Void
            }
            Ast::ArrayLiteral(elements) => {
                if elements.is_empty() {
                    panic!("Cannot infer type from an empty array");
                }
                let types: Vec<Type> = elements.iter().map(|x| self.check(x)).collect();
                let array_type = types
                    .iter()
                    .reduce(|lhs, rhs| {
                        StaticTypeChecker::assert_type(lhs.clone(), rhs.clone());
                        &rhs
                    })
                    .unwrap();
                Type::Array {
                    element_type: Box::new(array_type.clone()),
                }
            }
            Ast::ArrayLength(expr) => {
                if let Type::Array { element_type: _ } = self.check(expr) {
                    Type::Number
                } else {
                    panic!("Expected an array, but got {:?}", self.check(expr));
                }
            }
            Ast::ArrayLookup(array, index) => {
                StaticTypeChecker::assert_type(Type::Number, self.check(index));
                if let Type::Array { element_type: _ } = self.check(array) {
                    Type::Number
                } else {
                    panic!("Expected an array, but got {:?}", self.check(array));
                }
            }
            Ast::Function(name, function_type, block) => {
                self.functions.insert(name.clone(), function_type.clone());
                let (parameters, rt) = match function_type {
                    Type::Function {
                        parameter_types: ps,
                        return_type: rt,
                    } => (ps, rt),
                    _ => unreachable!(),
                };
                let mut env = StaticTypeChecker::new(
                    parameters.clone(),
                    self.functions.clone(),
                    Some(*rt.clone()),
                );
                env.check(block);
                Type::Void
            }
            Ast::Call(name, arguments) => {
                let called_f_signature = self
                    .functions
                    .clone()
                    .get(name)
                    .unwrap_or_else(|| panic!("Use of undefined function {}", name))
                    .clone();
                if let Type::Function {
                    parameter_types: ps,
                    return_type: rt,
                } = called_f_signature
                {
                    let arg_types: Vec<Type> = arguments.iter().map(|x| self.check(x)).collect();
                    let param_types: Vec<Type> = ps.iter().map(|(_, t)| t.clone()).collect();
                    for (arg, param) in arg_types.iter().zip(param_types) {
                        StaticTypeChecker::assert_type(arg.clone(), param);
                    }
                    *rt
                } else {
                    unreachable!()
                }
            }
            Ast::Return(expr) => {
                let t = self.check(expr);
                if let Some(rt) = self.current_return_type.clone() {
                    StaticTypeChecker::assert_type(rt, t);
                    Type::Void
                } else {
                    panic!("Return statement used outside of any function.");
                }
            }
            Ast::If(condition, consequence, alternative) => {
                self.check(condition);
                self.check(consequence);
                self.check(alternative);
                Type::Void
            }
            Ast::While(condition, body) => {
                self.check(condition);
                self.check(body);
                Type::Void
            }
            Ast::Block(statements) => {
                for statement in statements {
                    self.check(statement);
                }
                Type::Void
            }
        }
    }
}
