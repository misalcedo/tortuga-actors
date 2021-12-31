//! Parse a sequence of tokens into a syntax tree.

mod tokens;

use crate::compiler::errors::syntactical::ErrorKind;
use crate::compiler::parser::tokens::TokenMatcher;
use crate::compiler::{Kind, Token};
use crate::grammar::syntax::*;
use crate::{Scanner, SyntacticalError};
use std::iter::Peekable;
use std::str::FromStr;
use tokens::Tokens;

const COMPARISON_KINDS: &[Kind] = &[
    Kind::LessThan,
    Kind::GreaterThan,
    Kind::LessThanOrEqualTo,
    Kind::GreaterThanOrEqualTo,
    Kind::Equal,
    Kind::NotEqual,
];

/// A recursive descent LL(1) parser for the syntax grammar.
/// Parses a sequence of `Token`s into syntax tree.
pub struct Parser<T: Tokens> {
    tokens: T,
}

impl<'a> From<&'a str> for Parser<Peekable<Scanner<'a>>> {
    fn from(source: &'a str) -> Self {
        Parser {
            tokens: Scanner::from(source).peekable(),
        }
    }
}

impl<T: Tokens> From<T> for Parser<T> {
    fn from(tokens: T) -> Self {
        Parser { tokens }
    }
}

impl<T: Tokens> Parser<T> {
    /// Advances the token sequence and returns the next value if the token is one of the expected [`Kind`]s.
    ///
    /// Returns [`Err`] when at the end of the sequence,
    /// if the token's kind does not match, or if the token is invalid.
    fn next_kind<Matcher: TokenMatcher>(
        &mut self,
        matcher: Matcher,
    ) -> Result<Token, SyntacticalError> {
        if self.tokens.has_next() {
            match self.tokens.next_if_match(matcher) {
                Some(token) => Ok(token),
                None => Err(ErrorKind::NoMatch.into()),
            }
        } else {
            Err(ErrorKind::Incomplete.into())
        }
    }

    /// Generate a syntax tree rooted at a `Program` for this `Parser`'s sequence of tokens.
    pub fn parse(mut self) -> Result<Program, SyntacticalError> {
        let expression = self.parse_expression()?;

        match self
            .tokens
            .peek_kind()
            .ok_or_else(|| SyntacticalError::from(ErrorKind::Incomplete))?
        {
            Kind::LessThan
            | Kind::GreaterThan
            | Kind::LessThanOrEqualTo
            | Kind::GreaterThanOrEqualTo
            | Kind::Equal
            | Kind::NotEqual => self.parse_comparisons(expression),
            _ => self.parse_expressions(expression),
        }
    }

    fn parse_expressions(&mut self, expression: Expression) -> Result<Program, SyntacticalError> {
        let mut expressions = Vec::new();

        while self.tokens.has_next() {
            expressions.push(self.parse_expression()?);
        }

        Ok(List::new(expression, expressions).into())
    }

    fn parse_comparisons(&mut self, expression: Expression) -> Result<Program, SyntacticalError> {
        let head = self.parse_comparison()?;
        let mut comparisons = Vec::new();

        while self.tokens.has_next() {
            comparisons.push(self.parse_comparison()?);
        }

        Ok(Comparisons::new(expression, List::new(head, comparisons)).into())
    }

    fn parse_comparison(&mut self) -> Result<Comparison, SyntacticalError> {
        let operator = self.parse_comparison_operator(COMPARISON_KINDS)?;
        let expression = self.parse_expression()?;

        Ok(Comparison::new(operator, expression))
    }

    fn parse_comparison_operator(
        &mut self,
        kinds: &[Kind],
    ) -> Result<Comparator, SyntacticalError> {
        let operator = match self.next_kind(kinds)?.kind() {
            Kind::LessThan => Comparator::LessThan,
            Kind::GreaterThan => Comparator::GreaterThan,
            Kind::LessThanOrEqualTo => Comparator::LessThanOrEqualTo,
            Kind::GreaterThanOrEqualTo => Comparator::GreaterThanOrEqualTo,
            Kind::NotEqual => Comparator::NotEqualTo,
            _ => Comparator::EqualTo,
        };

        Ok(operator)
    }

    fn parse_expression(&mut self) -> Result<Expression, SyntacticalError> {
        if let Some(Kind::At) = self.tokens.peek_kind() {
            self.parse_assignment().map(Expression::from)
        } else {
            self.parse_epsilon().map(Expression::from)
        }
    }

    fn parse_epsilon(&mut self) -> Result<Epsilon, SyntacticalError> {
        let lhs = self.parse_modulo()?;
        let mut rhs = None;

        if self.tokens.next_if_match(&[Kind::Tilde]).is_some() {
            rhs = Some(self.parse_modulo()?);
        }

        Ok(Epsilon::new(lhs, rhs))
    }

    fn parse_modulo(&mut self) -> Result<Modulo, SyntacticalError> {
        Err(ErrorKind::NoMatch.into())
    }

    fn parse_sum(&mut self) -> Result<Sum, SyntacticalError> {
        Err(ErrorKind::NoMatch.into())
    }

    fn parse_product(&mut self) -> Result<Product, SyntacticalError> {
        Err(ErrorKind::NoMatch.into())
    }

    fn parse_power(&mut self) -> Result<Power, SyntacticalError> {
        let lhs = self.parse_primary()?;
        let mut rhs = Vec::new();

        while let Some(true) = self.tokens.has_next_match(Kind::Caret) {
            rhs.push(self.parse_primary()?);
        }

        Ok(List::new(lhs, rhs))
    }

    fn parse_primary(&mut self) -> Result<Primary, SyntacticalError> {
        match self.tokens.peek_kind() {
            Some(Kind::Minus | Kind::Number) => self.parse_number().map(Primary::from),
            Some(Kind::Identifier) => self.parse_call().map(Primary::from),
            Some(Kind::LeftParenthesis) => self.parse_grouping().map(Primary::from),
            Some(_) => Err(ErrorKind::NoMatch.into()),
            None => Err(ErrorKind::Incomplete.into()),
        }
    }

    fn parse_number(&mut self) -> Result<Number, SyntacticalError> {
        let negative = self.tokens.next_if_match(Kind::Minus).is_some();
        let number = self.next_kind(Kind::Number)?;

        Ok(Number::new(negative, *number.lexeme()))
    }

    fn parse_call(&mut self) -> Result<Call, SyntacticalError> {
        let identifier = self.next_kind(Kind::Identifier)?;

        self.next_kind(Kind::LeftParenthesis)?;

        let arguments = self.parse_arguments()?;

        self.next_kind(Kind::RightParenthesis)?;

        Ok(Call::new(*identifier.lexeme(), arguments))
    }

    fn parse_arguments(&mut self) -> Result<Arguments, SyntacticalError> {
        let head = self.parse_expression()?;
        let mut tail = Vec::new();

        while let Some(false) = self.tokens.has_next_match(Kind::RightParenthesis) {
            tail.push(self.parse_expression()?);
        }

        Ok(List::new(head, tail))
    }

    fn parse_grouping(&mut self) -> Result<Grouping, SyntacticalError> {
        self.next_kind(Kind::LeftParenthesis)?;

        let expression = self.parse_expression()?;

        self.next_kind(Kind::RightParenthesis)?;

        Ok(expression.into())
    }

    fn parse_assignment(&mut self) -> Result<Assignment, SyntacticalError> {
        Err(ErrorKind::NoMatch.into())
    }
}

impl FromStr for Program {
    type Err = SyntacticalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::from(s).parse()
    }
}