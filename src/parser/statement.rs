use crate::parser::combinators as cmb;
use crate::parser::combinators::OrValue;
use crate::parser::combinators::Parser;
use crate::parser::expression as exp;

use crate::ast::Node;

pub fn make_statement_parser<'a>() -> impl Parser<'a, Node> {
    |input: &'a str| {
        cmb::map(
            cmb::or(
                make_return_parser(),
                cmb::or(
                    make_function_parser(),
                    cmb::or(
                        make_if_parser(),
                        cmb::or(
                            make_while_parser(),
                            cmb::or(
                                make_var_parser(),
                                cmb::or(
                                    make_assignment_parser(),
                                    cmb::or(make_block_parser(), make_expression_parser()),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
            |return_or_other| match return_or_other {
                OrValue::Lhs(return_node) => return_node,
                OrValue::Rhs(if_or_other) => match if_or_other {
                    OrValue::Lhs(if_node) => if_node,
                    OrValue::Rhs(function_or_other) => match function_or_other {
                        OrValue::Lhs(function_node) => function_node,
                        OrValue::Rhs(while_or_other) => match while_or_other {
                            OrValue::Lhs(while_node) => while_node,
                            OrValue::Rhs(var_or_other) => match var_or_other {
                                OrValue::Lhs(var_node) => var_node,
                                OrValue::Rhs(assignment_or_other) => match assignment_or_other {
                                    OrValue::Lhs(assignment_node) => assignment_node,
                                    OrValue::Rhs(block_or_expr) => block_or_expr.extract().clone(),
                                },
                            },
                        },
                    },
                },
            },
        )
        .parse(input)

        //cmb::map(cmb::or(make_return_parser(), make_if_parser()),
        //|x| x.extract().clone()
        //).parse(input)
    }
}

// return_statement <- RETURN expression SEMICOLOn
pub fn make_return_parser<'a>() -> impl Parser<'a, Node> {
    cmb::and(
        exp::make_return_parser(),
        cmb::bind(exp::make_expression_parser(), |expr| {
            cmb::and(
                exp::make_semicolon_parser(),
                cmb::constant(Node::Return(Box::new(expr))),
            )
        }),
    )
}

// expression_statement <- expression SEMICOLON
pub fn make_expression_parser<'a>() -> impl Parser<'a, Node> {
    cmb::bind(exp::make_expression_parser(), |expr| {
        cmb::and(exp::make_semicolon_parser(), cmb::constant(expr))
    })
}

// if_statement <- IF LEFT_PAREN expression RIGHT_PAREN statement ELSE statement
pub fn make_if_parser<'a>() -> impl Parser<'a, Node> {
    cmb::and(
        exp::make_if_parser(),
        cmb::and(
            exp::make_left_paren_parser(),
            cmb::bind(exp::make_expression_parser(), move |conditional| {
                cmb::and(
                    exp::make_right_paren_parser(),
                    cmb::bind(make_statement_parser(), move |consequence| {
                        let conditional = conditional.clone();
                        cmb::and(
                            exp::make_else_parser(),
                            cmb::bind(make_statement_parser(), move |alternative| {
                                cmb::constant(Node::If(
                                    Box::new(conditional.clone()),
                                    Box::new(consequence.clone()),
                                    Box::new(alternative),
                                ))
                            }),
                        )
                    }),
                )
            }),
        ),
    )
}

// while_statement <- WHILE LEFT_PAREN expression RIGHT_PAREN statement
pub fn make_while_parser<'a>() -> impl Parser<'a, Node> {
    cmb::and(
        cmb::and(exp::make_while_parser(), exp::make_left_paren_parser()),
        cmb::bind(exp::make_expression_parser(), move |conditional| {
            cmb::and(
                exp::make_right_paren_parser(),
                cmb::bind(make_statement_parser(), move |stmt| {
                    cmb::constant(Node::While(Box::new(conditional.clone()), Box::new(stmt)))
                }),
            )
        }),
    )
}

// var_statement <- VAR ID ASSIGN expression SEMICOLON
pub fn make_var_parser<'a>() -> impl Parser<'a, Node> {
    cmb::and(
        exp::make_var_parser(),
        cmb::bind(exp::make_id_string_parser(), move |identifier| {
            cmb::and(
                exp::make_assign_parser(),
                cmb::bind(exp::make_expression_parser(), move |expr| {
                    cmb::and(
                        exp::make_semicolon_parser(),
                        cmb::constant(Node::Var(identifier.clone(), Box::new(expr))),
                    )
                }),
            )
        }),
    )
}

// assignment_statement <- ID ASSIGN expression SEMICOLON
pub fn make_assignment_parser<'a>() -> impl Parser<'a, Node> {
    cmb::bind(exp::make_id_string_parser(), move |identifier| {
        cmb::and(
            exp::make_assign_parser(),
            cmb::bind(exp::make_expression_parser(), move |expr| {
                cmb::and(
                    exp::make_semicolon_parser(),
                    cmb::constant(Node::Assignment(identifier.clone(), Box::new(expr))),
                )
            }),
        )
    })
}

// block_statement <- LEFT_BRACE statement* RIGHT_BRACE
pub fn make_block_parser<'a>() -> impl Parser<'a, Node> {
    cmb::and(
        exp::make_left_brace_parser(),
        cmb::bind(
            cmb::zero_or_more(make_statement_parser()),
            move |statements| {
                cmb::and(
                    exp::make_right_brace_parser(),
                    cmb::constant(Node::Block(statements)),
                )
            },
        ),
    )
}

// parameters_statement <- (ID (COMMA ID)*)?
pub fn make_parameters_parser<'a>() -> impl Parser<'a, Vec<String>> {
    cmb::bind(exp::make_id_string_parser(), move |first_id| {
        cmb::bind(
            cmb::zero_or_more(cmb::and(
                exp::make_comma_parser(),
                exp::make_id_string_parser(),
            )),
            move |rest_ids| {
                cmb::constant({
                    let mut v = vec![first_id.clone()];
                    v.extend(rest_ids);
                    v
                })
            },
        )
    })
}

// function_statement <- FUNCTION ID LEFT_PAREN paramters RIGHT_PAREN block_statement
pub fn make_function_parser<'a>() -> impl Parser<'a, Node> {
    cmb::and(
        exp::make_function_parser(),
        cmb::bind(exp::make_id_string_parser(), move |function_id| {
            cmb::and(
                exp::make_left_paren_parser(),
                cmb::bind(make_parameters_parser(), move |parameters| {
                    let function_id = function_id.clone();
                    cmb::and(
                        exp::make_right_paren_parser(),
                        cmb::bind(make_block_parser(), move |block| {
                            cmb::constant(Node::Function(
                                function_id.clone(),
                                parameters.clone(),
                                Box::new(block),
                            ))
                        }),
                    )
                }),
            )
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_parser() {
        let input = "return 1; //xx";
        let parser = make_return_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(parsed, Node::Return(Box::new(Node::Number(1))));
    }

    #[test]
    fn expression_parser() {
        let input = "1; //xx";
        let parser = make_expression_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(parsed, Node::Number(1));
    }

    #[test]
    fn if_parser() {
        let input = "if (1) 2; else 3; //xx";
        let parser = make_if_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Node::If(
                Box::new(Node::Number(1)),
                Box::new(Node::Number(2)),
                Box::new(Node::Number(3)),
            )
        );
    }

    #[test]
    fn while_parser() {
        let input = "while (1) 2; //xx";
        let parser = make_while_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Node::While(Box::new(Node::Number(1)), Box::new(Node::Number(2)),)
        );
    }

    #[test]
    fn var_parser() {
        let input = "var x = 1; //xx";
        let parser = make_var_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Node::Var(String::from("x"), Box::new(Node::Number(1)))
        );
    }

    #[test]
    fn assignment_parser() {
        let input = "x = 1; //xx";
        let parser = make_assignment_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Node::Assignment(String::from("x"), Box::new(Node::Number(1)))
        );
    }

    #[test]
    fn block_parser() {
        let input = "{1;2;} //xx";
        let parser = make_block_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(parsed, Node::Block(vec![Node::Number(1), Node::Number(2),]));
    }

    #[test]
    fn parameters_parser() {
        let input = "x, y, z //xx";
        let parser = make_parameters_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(parsed, vec!["x", "y", "z"]);
    }

    #[test]
    fn function_parser() {
        let input = "function f(x, y, z) { 1; } //xx";
        let parser = make_function_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Node::Function(
                String::from("f"),
                vec![String::from("x"), String::from("y"), String::from("z"),],
                Box::new(Node::Block(vec![Node::Number(1)]))
            )
        );
    }
}
