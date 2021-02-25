pub mod combinators;

use regex::Regex;

use combinators::Parsed;

#[derive(Debug, Clone, PartialEq)]
pub struct Source {
    remaining: String,
}

impl Source {
    pub fn new(source: String) -> Source {
        Source { remaining: source }
    }

    pub fn match_regex(self, regex: &Regex) -> Option<Parsed<String>> {
        // We should always try to match on the beginning of the source string
        assert!(regex.as_str().chars().take(1).next().unwrap() == '^');

        match regex.find(&self.remaining) {
            Some(m) => Some((
                String::from(m.as_str()),
                Source {
                    remaining: self.remaining[m.as_str().len()..].to_string(),
                },
            )),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::Source;

    #[test]
    fn matching_regex_succeeds() {
        let source = Source {
            remaining: String::from("abc123"),
        };
        let result = source.match_regex(&Regex::new("^ab").unwrap());
        let (matched_string, new_source) = result.unwrap();

        assert_eq!(matched_string, "ab");
        assert_eq!(new_source.remaining, "c123");
    }

    #[test]
    fn matching_regex_fails() {
        let source = Source {
            remaining: String::from("abc123"),
        };

        let result = source.match_regex(&Regex::new("^xd").unwrap());

        assert!(result.is_none());
    }
}
