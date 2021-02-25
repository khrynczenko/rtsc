use std::rc::Rc;

use regex::Regex;

use super::Source;

pub type Parsed<T> = (T, Source);

type RcParsingFunction<T> = Rc<dyn Fn(Source) -> Option<Parsed<T>>>;

pub trait TParser<T> {
    fn parse(&self, source: Source) -> Option<Parsed<T>>;
}

#[derive(Clone)]
pub struct Parser<T> {
    function: RcParsingFunction<T>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for Parser<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Parser<T> { function: Source -> T }")
    }
}

#[derive(Debug, Clone)]
pub enum OrValue<T, U> {
    Lhs(T),
    Rhs(U),
}

impl<T, U> PartialEq for OrValue<T, U>
where
    T: PartialEq,
    U: PartialEq,
{
    fn eq(&self, rhs: &OrValue<T, U>) -> bool {
        match (self, rhs) {
            (OrValue::Lhs(x), OrValue::Lhs(y)) => x == y,
            (OrValue::Rhs(x), OrValue::Rhs(y)) => x == y,
            (_, _) => false,
        }
    }
}

impl<T: 'static> Parser<T> {
    /// Creates a parser from a parsing function.
    pub fn new(function: RcParsingFunction<T>) -> Parser<T> {
        Parser { function }
    }

    /// Create a parser that produces a constant value without
    /// advancing the source.
    pub fn constant<U: Clone + 'static>(value: U) -> Parser<U> {
        Parser::new(Rc::new(move |source| Some((value.clone(), source))))
    }

    /// Creates a parser from a given regular expression.
    pub fn regex(regex: Regex) -> Parser<String> {
        Parser::new(Rc::new(move |source| source.match_regex(&regex)))
    }

    /// Given a parser creates a parser that can match zero or more times
    /// using the aformentioned given parser.
    pub fn zero_or_more(parser: Parser<T>) -> Parser<Vec<T>> {
        Parser::new(Rc::new(move |source| {
            let mut results = Vec::new();
            let mut new_source = source;
            loop {
                let result = parser.parse(new_source.clone());
                if result.is_none() {
                    return Some((results, new_source));
                }
                let (v, s) = result.unwrap();
                results.push(v);
                new_source = s;
            }
        }))
    }

    /// Creates a parser that panics when used.
    pub fn panic() -> Parser<()> {
        panic!();
    }

    /// Creates a parser from two different parsers and tries to match on
    /// the first one. If first one does not match tries the second one.
    pub fn or<U: 'static>(self, rhs: Parser<U>) -> Parser<OrValue<T, U>> {
        Parser::new(Rc::new(move |source| {
            let parsed = self.parse(source.clone());
            match parsed {
                Some((value, source)) => Some((OrValue::Lhs(value), source)),
                _ => {
                    let (value, source) = rhs.parse(source)?;
                    Some((OrValue::Rhs(value), source))
                }
            }
        }))
    }

    /// Creates a parser from two different parsers and tries to match on
    /// the first one and then the scond one. Both must match in that order.
    pub fn and<U: Clone + 'static>(self, rhs: Parser<U>) -> Parser<U> {
        self.bind(Rc::new(move |_| rhs.clone()))
    }

    /// Replaces the value produced by a parser.
    pub fn map<U: Clone + 'static>(self, callback: Rc<dyn Fn(T) -> U>) -> Parser<U> {
        Parser::new(Rc::new(move |source| {
            self.parse(source).map(|(v, s)| (callback(v), s))
        }))
    }

    /// Binds value to a parser.
    pub fn bind<U: 'static>(self, callback: Rc<dyn Fn(T) -> Parser<U>>) -> Parser<U> {
        Parser::new(Rc::new(move |source| {
            let (value, new_source) = self.parse(source)?;
            callback(value).parse(new_source)
        }))
    }

    pub fn maybe<U: 'static>(self, rhs: Parser<U>) -> Parser<Option<U>> {
        Parser::new(Rc::new(move |source| {
            rhs.parse(source.clone())
                .map_or(Some((None, source)), |(x, new_source)| {
                    Some((Some(x), new_source))
                })
        }))
    }

    pub fn parse_string_to_completion(self, string: String) -> Option<T> {
        let source = Source { remaining: string };
        let result = self.parse(source);
        result.map(|(x, _)| x)
    }
}

impl<T> TParser<T> for Parser<T> {
    fn parse(&self, source: Source) -> Option<Parsed<T>> {
        (self.function)(source)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use regex::Regex;

    use super::{OrValue, Parsed, Parser, Source, TParser};
    use crate::ast::Node;

    fn parse_identifier(source: Source) -> Option<Parsed<Node>> {
        let (name, source) = source.match_regex(&Regex::new(r"^[a-zA-Z][a-zA-Z\d]*").unwrap())?;
        Some((Node::Identifier(name), source))
    }

    #[test]
    fn parser_from_function_suceeds() {
        let source = Source {
            remaining: String::from("abc123"),
        };
        let parser = Parser::new(Rc::new(parse_identifier));
        let (node, new_source) = parser.parse(source).unwrap();

        assert_eq!(node, Node::Identifier(String::from("abc123")));
        assert_eq!(new_source.remaining, "");
    }

    #[test]
    fn parser_from_function_fails() {
        let source = Source {
            remaining: String::from("123"),
        };
        let parser = Parser::new(Rc::new(parse_identifier));

        assert!(parser.parse(source).is_none());
    }

    #[test]
    fn parser_from_regex_succeeds() {
        let source = Source {
            remaining: String::from("abc123"),
        };
        let parser = Parser::<String>::regex(Regex::new("^abc").unwrap());
        let (node, new_source) = parser.parse(source).unwrap();

        assert_eq!(node, String::from("abc"));
        assert_eq!(new_source.remaining, "123");
    }

    #[test]
    fn parser_from_regex_fails() {
        let source = Source {
            remaining: String::from("abc123"),
        };
        let parser = Parser::<String>::regex(Regex::new("^123").unwrap());
        let result = parser.parse(source);

        assert!(result.is_none());
    }

    #[test]
    fn parser_from_constant_succeeds() {
        let source = Source {
            remaining: String::from("abc123"),
        };
        let parser = Parser::<u32>::constant(1);
        let (value, new_source) = parser.parse(source).unwrap();

        assert_eq!(value, 1);
        assert_eq!(new_source.remaining, "abc123");
    }

    #[test]
    #[should_panic]
    fn parser_from_panic_panics() {
        Parser::<()>::panic();
    }

    #[test]
    fn parser_or_parser_succeeds() {
        let source = Source {
            remaining: String::from("abc123"),
        };
        let parser1 = Parser::<String>::regex(Regex::new("^1").unwrap());
        let parser2 = Parser::<u32>::constant(2);
        let or = parser1.or(parser2);
        let (value, new_source) = or.parse(source).unwrap();
        match value {
            OrValue::Lhs(_) => assert!(false),
            OrValue::Rhs(v) => assert_eq!(v, 2),
        }
        assert_eq!(new_source.remaining, "abc123");
    }

    #[test]
    fn parser_and_succeeds() {
        let source = Source {
            remaining: String::from("abc123"),
        };
        let parser1 = Parser::<String>::regex(Regex::new("^abc").unwrap());
        let parser2 = Parser::<String>::regex(Regex::new("^123").unwrap());
        let and = parser1.and(parser2);
        let (value, new_source) = and.parse(source).unwrap();

        assert_eq!(value, "123");
        assert_eq!(new_source.remaining, "");
    }

    #[test]
    fn parser_and_fails() {
        let source = Source {
            remaining: String::from("abc123"),
        };
        let parser1 = Parser::<String>::regex(Regex::new("^1").unwrap());
        let parser2 = Parser::<u32>::constant(2);
        let and = parser1.and(parser2);

        assert!(and.parse(source).is_none());
    }

    #[test]
    fn parser_zero_or_more_suceeds() {
        let source = Source {
            remaining: String::from("111abc123"),
        };
        let parser = Parser::<String>::regex(Regex::new("^1").unwrap());
        let zero_or_more = Parser::zero_or_more(parser);
        let (values, new_source) = zero_or_more.parse(source).unwrap();
        assert_eq!(values, vec!["1", "1", "1"]);
        assert_eq!(new_source.remaining, "abc123");
    }

    #[test]
    fn parser_map_succeeds() {
        let source = Source {
            remaining: String::from("abc123"),
        };
        let parser = Parser::<String>::regex(Regex::new("^[a-z]*").unwrap());
        let uppercase_parser =
            parser.map(Rc::new(|letters: String| letters.as_str().to_uppercase()));
        let (values, new_source) = uppercase_parser.parse(source).unwrap();
        assert_eq!(values, "ABC");
        assert_eq!(new_source.remaining, "123");
    }

    #[test]
    fn parser_binding_suceeds() {
        let source = Source {
            remaining: String::from("111abc123"),
        };
        let parser = Parser::<String>::regex(Regex::new("^1").unwrap());
        let zero_or_more = Parser::zero_or_more(parser);
        let ones_parser = zero_or_more.bind(Rc::new(|values: Vec<String>| {
            let concatenated: String = values.iter().fold(String::new(), |acc, item| acc + item);
            Parser::<i32>::constant(concatenated.parse::<i32>().unwrap())
        }));
        let (value, new_source) = ones_parser.parse(source).unwrap();

        assert_eq!(value, 111);
        assert_eq!(new_source.remaining, "abc123");
    }
}