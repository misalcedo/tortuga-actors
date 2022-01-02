//! Errors that may occur during lexical analysis.

use crate::compiler::Lexeme;
use crate::WithLexeme;
use std::fmt;
use std::fmt::{Display, Formatter};

/// An error that occurred during lexical analysis of a specific lexeme.
/// After an error is encountered, the scanner may continue to analyze the lexeme.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LexicalError {
    lexeme: Lexeme,
    kind: ErrorKind,
}

impl Display for LexicalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Encountered a {} error during lexical analysis on {}",
            self.kind, self.lexeme
        )
    }
}

impl std::error::Error for LexicalError {}

impl WithLexeme for LexicalError {
    fn lexeme(&self) -> &Lexeme {
        &self.lexeme
    }
}

/// The kind of lexical error that occurred.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Number,
    Invalid,
}

impl LexicalError {
    /// Creates a new instance of a `LexicalError`.
    pub fn new<L: Into<Lexeme>>(lexeme: L, kind: ErrorKind) -> Self {
        LexicalError {
            lexeme: lexeme.into(),
            kind,
        }
    }

    /// The actual text this lexical error represents in the input.
    pub fn lexeme(&self) -> &Lexeme {
        &self.lexeme
    }

    /// This `LexicalError`'s variant.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Number => f.write_str("NUMBER"),
            ErrorKind::Invalid => f.write_str("INVALID"),
        }
    }
}
