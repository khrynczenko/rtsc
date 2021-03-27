pub mod combinators;

mod expression;
mod statement;

use crate::ast::Ast;
use crate::parser::combinators as cmb;
use crate::parser::expression as exp;
use crate::parser::statement as stmt;
use combinators::Parser;

pub fn make_full_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::map(
        cmb::and(
            exp::make_ignored_parser(),
            cmb::zero_or_more(stmt::make_statement_parser()),
        ),
        Ast::Block,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_parser() {
        let input = "function factorial(n) {
                var result = 1;
                while (n != 1) {
                    result = result * n;
                    n = n - 1;
                }
                return result;
            } //xx
            ";
        let parser = make_full_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Ast::Block(vec![Ast::Function(
                String::from("factorial"),
                vec![String::from("n")],
                Box::new(Ast::Block(vec![
                    Ast::Var(String::from("result"), Box::new(Ast::Number(1))),
                    Ast::While(
                        Box::new(Ast::NotEqual(
                            Box::new(Ast::Identifier(String::from("n"))),
                            Box::new(Ast::Number(1))
                        )),
                        Box::new(Ast::Block(vec![
                            Ast::Assignment(
                                String::from("result"),
                                Box::new(Ast::Multiplication(
                                    Box::new(Ast::Identifier(String::from("result"))),
                                    Box::new(Ast::Identifier(String::from("n"))),
                                ))
                            ),
                            Ast::Assignment(
                                String::from("n"),
                                Box::new(Ast::Subtraction(
                                    Box::new(Ast::Identifier(String::from("n"))),
                                    Box::new(Ast::Number(1)),
                                ))
                            ),
                        ])),
                    ),
                    Ast::Return(Box::new(Ast::Identifier(String::from("result"))))
                ]))
            )])
        );
    }
}
