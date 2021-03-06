use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum OrValue<LHS, RHS> {
    Lhs(LHS),
    Rhs(RHS),
}

type ParseResult<'input, OutputT> = Result<(&'input str, OutputT), &'input str>;

pub trait Parser<'input, OutputT> {
    fn parse(&self, input: &'input str) -> ParseResult<'input, OutputT>;
}

impl<'input, F, OutputT> Parser<'input, OutputT> for F
where
    F: Fn(&'input str) -> ParseResult<'input, OutputT>,
{
    fn parse(&self, input: &'input str) -> ParseResult<'input, OutputT> {
        self(input)
    }
}

pub fn match_regex<'input>(input: &'input str, regex: &Regex) -> ParseResult<'input, String> {
    // We should always try to match on the beginning of the source string
    assert!(regex.as_str().chars().take(1).next().unwrap() == '^');

    match regex.find(input) {
        Some(matched) => Ok((
            &input[matched.as_str().len()..],
            matched.as_str().to_owned(),
        )),
        _ => Err(input),
    }
}

pub fn regex<'a>(regex: Regex) -> impl Parser<'a, String> {
    move |input: &'a str| match_regex(input, &regex)
}

pub fn constant<'a, T>(value: T) -> impl Parser<'a, T>
where
    T: Clone,
{
    move |input: &'a str| Ok((input, value.clone()))
}

pub fn error<'a>(message: &'a str) -> impl Parser<'a, &'a str> {
    move |_input: &'a str| Err(message)
}

pub fn maybe<'a, T>(parser: impl Parser<'a, T>) -> impl Parser<'a, Option<T>> {
    move |input| match parser.parse(input) {
        Ok((next_input, value)) => Ok((next_input, Some(value))),
        Err(_err) => Ok((input, None)),
    }
}

pub fn or<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, OrValue<R1, R2>>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    move |input: &'a str| {
        if let Ok((next_input, result1)) = parser1.parse(input) {
            return Ok((next_input, OrValue::Lhs(result1)));
        }

        if let Ok((next_input, result2)) = parser2.parse(input) {
            return Ok((next_input, OrValue::Rhs(result2)));
        }

        Err(input)
    }
}

pub fn and<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R2>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    move |input: &'a str| {
        if let Ok((next_input, _)) = parser1.parse(input) {
            if let Ok((next_input, result2)) = parser2.parse(next_input) {
                Ok((next_input, result2))
            } else {
                Err(input)
            }
        } else {
            Err(input)
        }
    }
}

pub fn zero_or_more<'a, T>(parser: impl Parser<'a, T>) -> impl Parser<'a, Vec<T>> {
    move |input| {
        let mut items = Vec::new();
        let mut input = input;
        while let Ok((next_input, item)) = parser.parse(input) {
            items.push(item);
            input = next_input;
        }
        Ok((input, items))
    }
}

pub fn bind<'a, F, A, B, NextParser>(parser: impl Parser<'a, A>, f: F) -> impl Parser<'a, B>
where
    NextParser: Parser<'a, B>,
    F: Fn(A) -> NextParser,
{
    move |input| match parser.parse(input) {
        Ok((next_input, result)) => f(result).parse(next_input),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_regex() {
        let regex = Regex::new("^123").unwrap();
        let input = "12345";
        assert_eq!(match_regex(input, &regex), Ok(("45", String::from("123"))));
    }

    #[test]
    fn constant_parser() {
        let input = "12345";
        assert_eq!(constant("123").parse(input), Ok(("12345", "123")));
    }

    #[test]
    fn error_parser() {
        let message = "error";
        let input = "12345";
        assert_eq!(error(message).parse(input), Err(message));
    }

    #[test]
    fn or_parser() {
        let error = error("");
        let constant1 = constant("123");
        let input = "12345";
        assert_eq!(
            or(error, constant1).parse(input),
            Ok(("12345", OrValue::Rhs("123")))
        );

        let constant2 = constant("1");
        let constant3 = constant("2");
        assert_eq!(
            or(constant2, constant3).parse(input),
            Ok(("12345", OrValue::Lhs("1")))
        );
    }

    #[test]
    fn and_parser() {
        let input = "12345";
        let regex1 = Regex::new("^12").unwrap();
        let regex2 = Regex::new("^3").unwrap();
        let p1 = regex(regex1);
        let p2 = regex(regex2);
        assert_eq!(and(p1, p2).parse(input), Ok(("45", String::from("3"))));

        let input = "12345";
        let regex3 = Regex::new("^12").unwrap();
        let regex4 = Regex::new("^4").unwrap();
        let p3 = regex(regex3);
        let p4 = regex(regex4);
        assert_eq!(and(p3, p4).parse(input), Err("12345"));
    }

    #[test]
    fn zero_or_more_parser() {
        let input = "1111END";
        let pattern = Regex::new("^1").unwrap();
        let one = regex(pattern);
        let parser = zero_or_more(one);

        assert_eq!(
            parser.parse(input),
            Ok((
                "END",
                vec!["1", "1", "1", "1"]
                    .into_iter()
                    .map(|s| String::from(s))
                    .collect()
            ))
        );
    }

    #[test]
    fn bind_parser() {
        let input = "1111END";
        let pattern_ones = Regex::new("^1111").unwrap();
        let ones = regex(pattern_ones);
        let pattern_end = Regex::new("^END").unwrap();
        let end = regex(pattern_end);

        let parser = bind(ones, |_ones| |input| end.parse(input));
        let result = parser.parse(input);

        assert_eq!(result, Ok(("", String::from("END"))));
    }

    #[test]
    fn maybe_parser() {
        let input = "1111END";
        let pattern_ones = Regex::new("^1112").unwrap();
        let ones = regex(pattern_ones);

        let parser = maybe(ones);
        let result = parser.parse(input);

        assert_eq!(result, Ok(("1111END", None)));

        let pattern_ones = Regex::new("^1111").unwrap();
        let ones_correct = regex(pattern_ones);
        let parser = maybe(ones_correct);
        let result = parser.parse(input);

        assert_eq!(result, Ok(("END", Some(String::from("1111")))));
    }
}
