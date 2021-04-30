use crate::parser::combinators as cmb;
use crate::parser::combinators::Parser;
use crate::parser::expression as exp;

use crate::ast::Ast;
use crate::types::Type;

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

pub fn make_optional_type_annotation_parser<'a>() -> impl Parser<'a, Option<Type>> {
    cmb::maybe(cmb::and(exp::make_colon_parser(), exp::make_type_parser()))
}

pub fn make_parameter_parser<'a>() -> impl Parser<'a, (String, Option<Type>)> {
    cmb::bind(exp::make_id_string_parser(), move |name| {
        cmb::bind(make_optional_type_annotation_parser(), move |opt_type| {
            cmb::constant((name.clone(), opt_type))
        })
    })
}

pub fn make_parameters_parser<'a>() -> impl Parser<'a, Vec<(String, Type)>> {
    cmb::bind(
        cmb::maybe(make_parameter_parser()),
        move |opt_first_param| {
            cmb::bind(
                cmb::zero_or_more(cmb::and(exp::make_comma_parser(), make_parameter_parser())),
                move |rest| {
                    if let Some((fst_name, fst_type)) = opt_first_param.clone() {
                        let mut output_params = vec![(fst_name, fst_type.unwrap_or(Type::Number))];
                        for param in rest {
                            output_params.push((param.0, param.1.unwrap_or(Type::Number)));
                        }
                        cmb::constant(output_params)
                    } else {
                        cmb::constant(vec![])
                    }
                },
            )
        },
    )
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
                    let parameters = parameters;
                    cmb::and(
                        exp::make_right_paren_parser(),
                        cmb::bind(
                            make_optional_type_annotation_parser(),
                            move |ret_type_annot| {
                                let parameters = parameters.clone();
                                let function_id = function_id.clone();
                                cmb::bind(make_block_parser(), move |block| {
                                    let type_ = Type::Function {
                                        parameter_types: parameters.iter().cloned().collect(),
                                        return_type: Box::new(
                                            ret_type_annot.clone().unwrap_or(Type::Number),
                                        ),
                                    };
                                    let f =
                                        Ast::Function(function_id.clone(), type_, Box::new(block));
                                    cmb::constant(f)
                                })
                            },
                        ),
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
        assert_eq!(
            parsed,
            vec![
                (String::from("x"), Type::Number),
                (String::from("y"), Type::Number),
                (String::from("z"), Type::Number)
            ]
        );
    }

    #[test]
    fn parameters_with_types_parser() {
        let input = "x: number, y: boolean, z: void , w: number//xx";
        let parser = make_parameters_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            vec![
                (String::from("x"), Type::Number),
                (String::from("y"), Type::Boolean),
                (String::from("z"), Type::Void),
                (String::from("w"), Type::Number)
            ]
        );
    }

    #[test]
    fn function_parser() {
        let input = "function f(x:number, y:boolean, z: number, w: boolean) { 1; } //xx";
        let parser = make_function_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Ast::Function(
                String::from("f"),
                Type::Function {
                    parameter_types: ([
                        (String::from("x"), Type::Number),
                        (String::from("y"), Type::Boolean),
                        (String::from("z"), Type::Number),
                        (String::from("w"), Type::Boolean)
                    ])
                    .iter()
                    .cloned()
                    .collect(),
                    return_type: Box::new(Type::Number)
                },
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
                Type::Function {
                    parameter_types: ([]).iter().cloned().collect(),
                    return_type: Box::new(Type::Number)
                },
                Box::new(Ast::Block(vec![Ast::Number(1)]))
            )
        );
    }
}
