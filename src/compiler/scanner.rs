//! Performs lexical analysis on Tortuga input and produces a sequence of `Token`s.

use crate::compiler::errors::lexical::ErrorKind;
use crate::compiler::unicode::UnicodeProperties;
use crate::compiler::{Input, Kind, LexicalError, Token};
use std::str::Chars;

/// A lexical analyzer with 1 character of lookahead.
#[derive(Clone, Debug)]
pub struct Scanner<'a> {
    source: &'a str,
    input: Input<Chars<'a>>,
}

impl<'a> From<&'a str> for Scanner<'a> {
    fn from(source: &'a str) -> Scanner<'a> {
        Scanner {
            source,
            input: source.into(),
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.input.skip_blank_space();

            let token = match self.input.next()? {
                '+' => Token::new(self.input.advance(), Kind::Plus),
                '-' => Token::new(self.input.advance(), Kind::Minus),
                '*' => Token::new(self.input.advance(), Kind::Star),
                '/' => Token::new(self.input.advance(), Kind::Slash),
                '^' => Token::new(self.input.advance(), Kind::Caret),
                '=' => Token::new(self.input.advance(), Kind::Equal),
                '~' => Token::new(self.input.advance(), Kind::Tilde),
                '%' => Token::new(self.input.advance(), Kind::Percent),
                '_' => Token::new(self.input.advance(), Kind::Underscore),
                '(' => Token::new(self.input.advance(), Kind::LeftParenthesis),
                ')' => Token::new(self.input.advance(), Kind::RightParenthesis),
                '[' => Token::new(self.input.advance(), Kind::LeftBracket),
                ']' => Token::new(self.input.advance(), Kind::RightBracket),
                '{' => Token::new(self.input.advance(), Kind::LeftBrace),
                '}' => Token::new(self.input.advance(), Kind::RightBrace),
                ',' => Token::new(self.input.advance(), Kind::Comma),
                ';' => {
                    self.skip_comment();
                    continue;
                }
                '<' => self.scan_less_than(),
                '>' => self.scan_greater_than(),
                '.' => self.scan_number(),
                d if d.is_ascii_digit() => self.scan_number(),
                s if s.is_xid_start() => self.scan_identifier(),
                _ => return self.scan_invalid(),
            };

            return Some(Ok(token));
        }
    }
}

impl<'a> Scanner<'a> {
    fn skip_comment(&mut self) {
        while self.input.next_unless_eq('\n').is_some() {}
    }

    fn scan_less_than(&mut self) -> Token {
        let kind = if self.input.next_if_eq('=').is_some() {
            Kind::LessThanOrEqualTo
        } else if self.input.next_if_eq('>').is_some() {
            Kind::NotEqual
        } else {
            Kind::LessThan
        };

        Token::new(self.input.advance(), kind)
    }

    fn scan_greater_than(&mut self) -> Token {
        let kind = if self.input.next_if_eq('=').is_some() {
            Kind::GreaterThanOrEqualTo
        } else {
            Kind::GreaterThan
        };

        Token::new(self.input.advance(), kind)
    }

    fn scan_number(&mut self) -> Token {
        while self.input.next_if(|c| c.is_ascii_digit()).is_some() {}
        Token::new(self.input.advance(), Kind::Number(42.into()))
    }

    fn scan_identifier(&mut self) -> Token {
        while self.input.next_if(|c| c.is_xid_continue()).is_some() {}
        Token::new(self.input.advance(), Kind::Identifier)
    }

    fn scan_invalid(&mut self) -> Option<Result<Token, LexicalError>> {
        while self
            .input
            .next_if(|c| {
                !c.is_ascii_punctuation()
                    && !c.is_ascii_digit()
                    && !c.is_xid_start()
                    && !c.is_pattern_white_space()
            })
            .is_some()
        {}

        Some(Err(LexicalError::new(
            self.input.advance(),
            ErrorKind::Invalid,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Lexeme, Location};

    fn validate(kind: Kind) {
        let input = kind.to_string();
        let mut scanner: Scanner<'_> = input.as_str().into();

        assert_eq!(
            scanner.next(),
            Some(Ok(Token::new(
                Lexeme::new(Location::default(), input.as_str()),
                kind
            )))
        );
        assert_eq!(scanner.next(), None);
    }

    #[test]
    fn scan_simple() {
        validate(Kind::Plus);
        validate(Kind::Minus);
        validate(Kind::Star);
        validate(Kind::Slash);
        validate(Kind::Percent);
        validate(Kind::Caret);
        validate(Kind::Tilde);
        validate(Kind::Equal);
        validate(Kind::NotEqual);
        validate(Kind::LessThan);
        validate(Kind::LessThanOrEqualTo);
        validate(Kind::GreaterThan);
        validate(Kind::GreaterThanOrEqualTo);
        validate(Kind::Comma);
        validate(Kind::Underscore);
        validate(Kind::LeftParenthesis);
        validate(Kind::RightParenthesis);
        validate(Kind::LeftBrace);
        validate(Kind::RightBrace);
        validate(Kind::LeftBracket);
        validate(Kind::RightBracket);
    }

    #[test]
    fn skips_invalid_characters() {
        let input = "\u{0E01EF}\u{0E01EF}\u{0E01EF}\u{0E01EF} +";
        let mut scanner: Scanner<'_> = input.into();

        let bad = Location::from(&input[..input.len() - 2]);

        assert_eq!(
            scanner.next(),
            Some(Err(LexicalError::new(
                Lexeme::new(Location::default(), bad),
                ErrorKind::Invalid
            )))
        );

        assert_eq!(
            scanner.next(),
            Some(Ok(Token::new(
                Lexeme::new(&input[..input.len() - 1], input),
                Kind::Plus
            )))
        );
        assert_eq!(scanner.next(), None);
    }

    fn validate_identifier(identifier: &str) {
        let mut scanner: Scanner<'_> = identifier.into();

        assert_eq!(
            scanner.next(),
            Some(Ok(Token::new(
                Lexeme::new(Location::default(), identifier),
                Kind::Identifier
            )))
        );
        assert_eq!(scanner.next(), None);
    }

    #[test]
    fn scan_identifier() {
        validate_identifier("x");
        validate_identifier("x2");
        validate_identifier("x_2");
        validate_identifier("x__2");
        validate_identifier("xx");
        validate_identifier("x__");
        validate_identifier("x_y_z");
        validate_identifier("x_y_z_");
    }

    fn validate_number(number: &str) {
        let mut scanner: Scanner<'_> = number.into();

        assert_eq!(
            scanner.next(),
            Some(Ok(Token::new(
                Lexeme::new(Location::default(), number),
                Kind::Number(42.into())
            )))
        );
        assert_eq!(scanner.next(), None);
    }

    #[test]
    fn scan_number() {
        validate_number("0");
        validate_number("2");
        validate_number("21");
        validate_number("100");
        validate_number(".100");
        validate_number(".5");
    }

    #[test]
    fn skip_comment() {
        let input = "; hello, world!\n \t42";

        assert_forty_two(input);
    }

    fn assert_forty_two(input: &str) {
        let mut scanner: Scanner<'_> = input.into();

        assert_eq!(
            scanner.next(),
            Some(Ok(Token::new(
                Lexeme::new(&input[..input.len() - 2], input),
                Kind::Number(42.into())
            )))
        );
        assert_eq!(scanner.next(), None);
    }

    #[test]
    fn skip_multiple_comments() {
        let input = "; hello, world!\n \t; foobar\n\n42";

        assert_forty_two(input);
    }

    #[test]
    fn scan_invalid_number() {
        todo!("0008 is not valid.")
    }

    #[test]
    fn scan_edge_cases() {
        todo!("2x is number then identifier.")
    }
}
