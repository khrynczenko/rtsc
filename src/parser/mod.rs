pub mod combinators;
mod expression;
mod statement;

use crate::parser::combinators as cmb;
use crate::parser::expression as exp;
use crate::parser::statement as stmt;
use combinators::Parser;

use crate::ast::Node;

pub fn make_full_parser<'a>() -> impl Parser<'a, Node> {
    cmb::map(
        cmb::and(
            exp::make_ignored_parser(),
            cmb::zero_or_more(stmt::make_statement_parser()),
        ),
        Node::Block,
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
            Node::Block(vec![Node::Function(
                String::from("factorial"),
                vec![String::from("n")],
                Box::new(Node::Block(vec![
                    Node::Var(String::from("result"), Box::new(Node::Number(1))),
                    Node::While(
                        Box::new(Node::NotEqual(
                            Box::new(Node::Identifier(String::from("n"))),
                            Box::new(Node::Number(1))
                        )),
                        Box::new(Node::Block(vec![
                            Node::Assignment(
                                String::from("result"),
                                Box::new(Node::Multiplication(
                                    Box::new(Node::Identifier(String::from("result"))),
                                    Box::new(Node::Identifier(String::from("n"))),
                                ))
                            ),
                            Node::Assignment(
                                String::from("n"),
                                Box::new(Node::Subtraction(
                                    Box::new(Node::Identifier(String::from("n"))),
                                    Box::new(Node::Number(1)),
                                ))
                            ),
                        ])),
                    ),
                    Node::Return(Box::new(Node::Identifier(String::from("result"))))
                ]))
            )])
        );
    }
}
