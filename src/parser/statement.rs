use crate::parser::combinators as cmb;
use crate::parser::combinators::Parser;
use crate::parser::expression as exp;

use crate::ast::Ast;

pub fn make_statement_parser<'a>() -> impl Parser<'a, Ast> {
    |input: &'a str| {
        let parser = cmb::or_(make_return_parser(), make_if_parser());
        let parser = cmb::or_(parser, make_while_parser());
        let parser = cmb::or_(parser, make_var_parser());
        let parser = cmb::or_(parser, make_assignment_parser());
        let parser = cmb::or_(parser, make_block_parser());
        let parser = cmb::or_(parser, make_function_parser());
        let parser = cmb::or_(parser, make_expression_parser());
        parser.parse(input)
    }
}

// return_statement <- RETURN expression SEMICOLOn
pub fn make_return_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::and(
        exp::make_return_parser(),
        cmb::bind(exp::make_expression_parser(), |expr| {
            cmb::and(
                exp::make_semicolon_parser(),
                cmb::constant(Ast::Return(Box::new(expr))),
            )
        }),
    )
}

// expression_statement <- expression SEMICOLON
pub fn make_expression_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::bind(exp::make_expression_parser(), |expr| {
        cmb::and(exp::make_semicolon_parser(), cmb::constant(expr))
    })
}

// if_statement <- IF LEFT_PAREN expression RIGHT_PAREN statement ELSE statement
pub fn make_if_parser<'a>() -> impl Parser<'a, Ast> {
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
                                cmb::constant(Ast::If(
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
pub fn make_while_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::and(
        cmb::and(exp::make_while_parser(), exp::make_left_paren_parser()),
        cmb::bind(exp::make_expression_parser(), move |conditional| {
            cmb::and(
                exp::make_right_paren_parser(),
                cmb::bind(make_statement_parser(), move |stmt| {
                    cmb::constant(Ast::While(Box::new(conditional.clone()), Box::new(stmt)))
                }),
            )
        }),
    )
}

// var_statement <- VAR ID ASSIGN expression SEMICOLON
pub fn make_var_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::and(
        exp::make_var_parser(),
        cmb::bind(exp::make_id_string_parser(), move |identifier| {
            cmb::and(
                exp::make_assign_parser(),
                cmb::bind(exp::make_expression_parser(), move |expr| {
                    cmb::and(
                        exp::make_semicolon_parser(),
                        cmb::constant(Ast::Var(identifier.clone(), Box::new(expr))),
                    )
                }),
            )
        }),
    )
}

// assignment_statement <- ID ASSIGN expression SEMICOLON
pub fn make_assignment_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::bind(exp::make_id_string_parser(), move |identifier| {
        cmb::and(
            exp::make_assign_parser(),
            cmb::bind(exp::make_expression_parser(), move |expr| {
                cmb::and(
                    exp::make_semicolon_parser(),
                    cmb::constant(Ast::Assignment(identifier.clone(), Box::new(expr))),
                )
            }),
        )
    })
}

// block_statement <- LEFT_BRACE statement* RIGHT_BRACE
pub fn make_block_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::and(
        exp::make_left_brace_parser(),
        cmb::bind(
            cmb::zero_or_more(make_statement_parser()),
            move |statements| {
                cmb::and(
                    exp::make_right_brace_parser(),
                    cmb::constant(Ast::Block(statements)),
                )
            },
        ),
    )
}

// parameters_statement <- (ID (COMMA ID)*)?
pub fn make_parameters_parser<'a>() -> impl Parser<'a, Option<Vec<String>>> {
    cmb::maybe(cmb::bind(exp::make_id_string_parser(), move |first_id| {
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
    }))
}

// function_statement <- FUNCTION ID LEFT_PAREN paramters RIGHT_PAREN block_statement
pub fn make_function_parser<'a>() -> impl Parser<'a, Ast> {
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
                            if let Some(ref params) = parameters {
                                cmb::constant(Ast::Function(
                                    function_id.clone(),
                                    params.clone(),
                                    Box::new(block),
                                ))
                            } else {
                                cmb::constant(Ast::Function(
                                    function_id.clone(),
                                    vec![],
                                    Box::new(block),
                                ))
                            }
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
        assert_eq!(parsed, Ast::Return(Box::new(Ast::Number(1))));
    }

    #[test]
    fn expression_parser() {
        let input = "1; //xx";
        let parser = make_expression_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(parsed, Ast::Number(1));
    }

    #[test]
    fn if_parser() {
        let input = "if (1) 2; else 3; //xx";
        let parser = make_if_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Ast::If(
                Box::new(Ast::Number(1)),
                Box::new(Ast::Number(2)),
                Box::new(Ast::Number(3)),
            )
        );
    }

    #[test]
    fn while_parser() {
        let input = "while (1) { 2; } //xx";
        let parser = make_while_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Ast::While(
                Box::new(Ast::Number(1)),
                Box::new(Ast::Block(vec![Ast::Number(2)]))
            )
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
            Ast::Var(String::from("x"), Box::new(Ast::Number(1)))
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
            Ast::Assignment(String::from("x"), Box::new(Ast::Number(1)))
        );
    }

    #[test]
    fn block_parser() {
        let input = "{1;2;} //xx";
        let parser = make_block_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(parsed, Ast::Block(vec![Ast::Number(1), Ast::Number(2),]));
    }

    #[test]
    fn parameters_parser() {
        let input = "x, y, z //xx";
        let parser = make_parameters_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(parsed.unwrap(), vec!["x", "y", "z"]);
    }

    #[test]
    fn function_parser() {
        let input = "function f(x, y, z) { 1; } //xx";
        let parser = make_function_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Ast::Function(
                String::from("f"),
                vec![String::from("x"), String::from("y"), String::from("z"),],
                Box::new(Ast::Block(vec![Ast::Number(1)]))
            )
        );
    }

    #[test]
    fn function_parser_without_args() {
        let input = "function f() { 1; } //xx";
        let parser = make_function_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Ast::Function(
                String::from("f"),
                vec![],
                Box::new(Ast::Block(vec![Ast::Number(1)]))
            )
        );
    }
}
