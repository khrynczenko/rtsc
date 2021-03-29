use regex::Regex;

use crate::parser::combinators as cmb;
use crate::parser::combinators::{OrValue, Parser};

use crate::ast::Ast;

pub type Comment = String;
pub type Whitespace = String;

macro_rules! token_parser {
    ($name:ident, $pattern:expr) => {
        #[allow(clippy::trivial_regex)]
        pub fn $name<'a>() -> impl Parser<'a, String> {
            make_token_parser(Regex::new($pattern).unwrap())
        }
    };
}

pub fn make_token_parser<'a>(pattern: Regex) -> impl Parser<'a, String> {
    let pattern_parser = cmb::regex(pattern);
    cmb::bind(pattern_parser, |value| {
        cmb::and(make_ignored_parser(), cmb::constant(value))
    })
}

token_parser! {make_function_parser, "^function"}
token_parser! {make_if_parser, "^if"}
token_parser! {make_else_parser, "^else"}
token_parser! {make_return_parser, "^return"}
token_parser! {make_while_parser, "^while"}
token_parser! {make_var_parser, "^var"}
token_parser! {make_assign_parser, "^="}
token_parser! {make_comma_parser, "^,"}
token_parser! {make_semicolon_parser, "^;"}
token_parser! {make_left_paren_parser, r"^\("}
token_parser! {make_right_paren_parser, r"^\)"}
token_parser! {make_left_brace_parser, r"^\{"}
token_parser! {make_right_brace_parser, r"^\}"}
token_parser! {make_not_parser, r"^!"}
token_parser! {make_equal_parser, r"^=="}
token_parser! {make_not_equal_parser, r"^!="}
token_parser! {make_plus_parser, r"^\+"}
token_parser! {make_minus_parser, r"^\-"}
token_parser! {make_star_parser, r"^\*"}
token_parser! {make_slash_parser, r"^/"}
token_parser! {make_id_string_parser, r"^[a-zA-Z_][a-zA-Z0-9_]*"}

pub fn make_expression_parser<'a>() -> impl Parser<'a, Ast> {
    |input| make_comparison_parser().parse(input)
}

pub fn make_ignored_parser<'a>() -> impl Parser<'a, Vec<OrValue<Whitespace, Comment>>> {
    let whitespace_regex = Regex::new(r"^[ \n\r\t]+").unwrap();
    let whitespace_parser = cmb::regex(whitespace_regex);
    let comment_regex = Regex::new(r"^//.*").unwrap();
    let comment_parser = cmb::regex(comment_regex);
    cmb::zero_or_more(cmb::or(whitespace_parser, comment_parser))
}

pub fn make_number_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::map(
        make_token_parser(Regex::new("^[0-9]+").unwrap()),
        |text: String| match text.parse::<i32>() {
            Ok(value) => Ast::Number(value),
            Err(_) => unreachable!(),
        },
    )
}

pub fn make_identifier_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::map(make_id_string_parser(), Ast::Identifier)
}

pub fn make_args_parser<'a>() -> impl Parser<'a, Vec<Ast>> {
    cmb::bind(cmb::maybe(make_expression_parser()), |arg| {
        cmb::bind(
            cmb::zero_or_more(cmb::and(make_comma_parser(), make_expression_parser())),
            move |ref mut args| {
                cmb::constant({
                    if let Some(ref first_arg) = arg {
                        let mut all_args = Vec::with_capacity(1 + args.len());
                        all_args.push(first_arg.clone());
                        all_args.append(args);
                        all_args
                    } else {
                        Vec::new()
                    }
                })
            },
        )
    })
}

pub fn make_call_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::bind(make_id_string_parser(), move |name| {
        cmb::bind(
            cmb::and(make_left_paren_parser(), make_args_parser()),
            move |args| {
                cmb::and(
                    make_right_paren_parser(),
                    cmb::constant(Ast::Call(name.clone(), args)),
                )
            },
        )
    })
}

// atom <- call | ID | NUMBER | LEFT_PAREN expression RIGHT_PAREN
pub fn make_atom_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::bind(
        cmb::or(
            make_call_parser(),
            cmb::or(
                make_identifier_parser(),
                cmb::or(
                    make_number_parser(),
                    cmb::and(
                        make_left_paren_parser(),
                        cmb::bind(make_expression_parser(), |e| {
                            cmb::and(make_right_paren_parser(), cmb::constant(e))
                        }),
                    ),
                ),
            ),
        ),
        |e| match e {
            OrValue::Lhs(call) => cmb::constant(call),
            OrValue::Rhs(id_or_number_or_expr) => match id_or_number_or_expr {
                OrValue::Lhs(id) => cmb::constant(id),
                OrValue::Rhs(number_or_expr) => match number_or_expr {
                    OrValue::Lhs(number) => cmb::constant(number),
                    OrValue::Rhs(expr) => cmb::constant(expr),
                },
            },
        },
    )
}
// unary <- NOT? atom
pub fn make_unary_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::bind(cmb::maybe(make_not_parser()), move |not| {
        cmb::map(make_atom_parser(), move |term| {
            if not.is_some() {
                Ast::Not(Box::new(term))
            } else {
                term
            }
        })
    })
}

// product <- unary ((STAR / SLASH) unary)*
pub fn make_product_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::bind(make_unary_parser(), move |first| {
        cmb::map(
            cmb::zero_or_more(cmb::bind(
                cmb::or(make_star_parser(), make_slash_parser()),
                move |operator| {
                    cmb::bind(make_unary_parser(), move |term| {
                        cmb::constant((operator.clone(), term))
                    })
                },
            )),
            move |operator_terms: Vec<(OrValue<String, String>, Ast)>| {
                operator_terms
                    .into_iter()
                    .fold(first.clone(), move |acc, (operator, term)| match operator {
                        OrValue::Lhs(_star) => Ast::Multiplication(Box::new(acc), Box::new(term)),
                        OrValue::Rhs(_slash) => Ast::Division(Box::new(acc), Box::new(term)),
                    })
            },
        )
    })
}

// sum <- product ((PLUS / MINUS) product)*
pub fn make_sum_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::bind(make_product_parser(), move |first| {
        cmb::map(
            cmb::zero_or_more(cmb::bind(
                cmb::or(make_plus_parser(), make_minus_parser()),
                move |operator| {
                    cmb::bind(make_product_parser(), move |term| {
                        cmb::constant((operator.clone(), term))
                    })
                },
            )),
            move |operator_terms: Vec<(OrValue<String, String>, Ast)>| {
                operator_terms
                    .into_iter()
                    .fold(first.clone(), move |acc, (operator, term)| match operator {
                        OrValue::Lhs(_plus) => Ast::Addition(Box::new(acc), Box::new(term)),
                        OrValue::Rhs(_minus) => Ast::Subtraction(Box::new(acc), Box::new(term)),
                    })
            },
        )
    })
}

// comparison <- sum ((EQUAL / NOT_EQUAL) sum)*
pub fn make_comparison_parser<'a>() -> impl Parser<'a, Ast> {
    cmb::bind(make_sum_parser(), move |first| {
        cmb::map(
            cmb::zero_or_more(cmb::bind(
                cmb::or(make_equal_parser(), make_not_equal_parser()),
                move |operator| {
                    cmb::bind(make_sum_parser(), move |term| {
                        cmb::constant((operator.clone(), term))
                    })
                },
            )),
            move |operator_terms: Vec<(OrValue<String, String>, Ast)>| {
                operator_terms
                    .into_iter()
                    .fold(first.clone(), move |acc, (operator, term)| match operator {
                        OrValue::Lhs(_equal) => Ast::Equal(Box::new(acc), Box::new(term)),
                        OrValue::Rhs(_not_equal) => Ast::NotEqual(Box::new(acc), Box::new(term)),
                    })
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignored_parser() {
        let input = "//line1\n\t\r  qweqwrasf";
        let parser = make_ignored_parser();
        let next_input = parser.parse(input).unwrap().0;
        let parsed = parser.parse(input).unwrap().1;
        assert_eq!(next_input, "qweqwrasf");
        assert_eq!(parsed[0], OrValue::Rhs(String::from("//line1")));
        assert_eq!(parsed[1], OrValue::Lhs(String::from("\n\t\r  ")));
    }

    #[test]
    fn token_parser() {
        let input = "fun  //xx";
        let parser = make_token_parser(Regex::new("^fun").unwrap());
        let next_input = parser.parse(input).unwrap().0;
        let parsed = parser.parse(input).unwrap().1;
        assert_eq!(next_input, "");
        assert_eq!(parsed, String::from("fun"));
    }

    #[test]
    fn number_parser() {
        let input = "123  //xx";
        let parser = make_number_parser();
        let next_input = parser.parse(input).unwrap().0;
        let parsed = parser.parse(input).unwrap().1;
        assert_eq!(next_input, "");
        assert_eq!(parsed, Ast::Number(123));
    }

    #[test]
    fn args_parser() {
        let input = "arg1, arg2, arg3  //xx";
        let parser = make_args_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            vec![
                Ast::Identifier(String::from("arg1")),
                Ast::Identifier(String::from("arg2")),
                Ast::Identifier(String::from("arg3"))
            ]
        );
    }

    #[test]
    fn call_parser_with_args() {
        let input = "f(arg1, arg2, arg3)  //xx";
        let parser = make_call_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        let (name, args) = match parsed {
            Ast::Call(name, args) => (name, args),
            _ => panic!(),
        };
        assert_eq!(next_input, "");
        assert_eq!(&name, "f");
        assert_eq!(
            args,
            vec![
                Ast::Identifier(String::from("arg1")),
                Ast::Identifier(String::from("arg2")),
                Ast::Identifier(String::from("arg3"))
            ]
        );
    }

    #[test]
    fn call_parser_without_args() {
        let input = "f()  //xx";
        let parser = make_call_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        let (name, args) = match parsed {
            Ast::Call(name, args) => (name, args),
            _ => panic!(),
        };
        assert_eq!(next_input, "");
        assert_eq!(&name, "f");
        assert_eq!(args, vec![]);
    }

    #[test]
    fn atom_parser_for_call() {
        let input = "f()  //xx";
        let parser = make_atom_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        let (name, args) = match parsed {
            Ast::Call(name, args) => (name, args),
            _ => panic!(),
        };
        assert_eq!(next_input, "");
        assert_eq!(&name, "f");
        assert_eq!(args, vec![]);
    }

    #[test]
    fn atom_parser_for_id() {
        let input = "identifier //xx";
        let parser = make_atom_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(parsed, Ast::Identifier(String::from("identifier")));
    }

    #[test]
    fn atom_parser_for_number() {
        let input = "123 //xx";
        let parser = make_atom_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(parsed, Ast::Number(123));
    }

    #[test]
    fn unary_parser() {
        let input = "!id //xx";
        let parser = make_unary_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Ast::Not(Box::new(Ast::Identifier(String::from("id"))))
        );
    }

    #[test]
    fn product_parser() {
        let input = "1 * 2 / 3 //xx";
        let parser = make_product_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Ast::Division(
                Box::new(Ast::Multiplication(
                    Box::new(Ast::Number(1)),
                    Box::new(Ast::Number(2))
                )),
                Box::new(Ast::Number(3))
            )
        );
    }

    #[test]
    fn sum_parser() {
        let input = "1 + 2 - 3 //xx";
        let parser = make_sum_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Ast::Subtraction(
                Box::new(Ast::Addition(
                    Box::new(Ast::Number(1)),
                    Box::new(Ast::Number(2))
                )),
                Box::new(Ast::Number(3))
            )
        );
    }

    #[test]
    fn comparison_parser() {
        let input = "1 == 2 != 3 //xx";
        let parser = make_comparison_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Ast::NotEqual(
                Box::new(Ast::Equal(
                    Box::new(Ast::Number(1)),
                    Box::new(Ast::Number(2))
                )),
                Box::new(Ast::Number(3))
            )
        );
    }

    #[test]
    fn expression_parser() {
        let input = "f(1 * 2) //xx";
        let parser = make_comparison_parser();
        let (next_input, parsed) = parser.parse(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(
            parsed,
            Ast::Call(
                String::from("f"),
                vec![Ast::Multiplication(
                    Box::new(Ast::Number(1)),
                    Box::new(Ast::Number(2))
                )],
            )
        );
    }
}