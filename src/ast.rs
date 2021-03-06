use crate::types::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ast {
    Null,
    Undefined,
    Number(i32),
    Bool(bool),
    ArrayLiteral(Vec<Ast>),
    ArrayLookup(Box<Ast>, Box<Ast>),
    ArrayLength(Box<Ast>),
    Identifier(String),
    Not(Box<Ast>),
    Equal(Box<Ast>, Box<Ast>),
    NotEqual(Box<Ast>, Box<Ast>),
    Addition(Box<Ast>, Box<Ast>),
    Subtraction(Box<Ast>, Box<Ast>),
    Multiplication(Box<Ast>, Box<Ast>),
    Division(Box<Ast>, Box<Ast>),
    Call(String, Vec<Ast>),
    Return(Box<Ast>),
    Block(Vec<Ast>),
    If(Box<Ast>, Box<Ast>, Box<Ast>),
    Function(String, Type, Box<Ast>),
    Var(String, Box<Ast>),
    Assignment(String, Box<Ast>),
    While(Box<Ast>, Box<Ast>),
}
