use std::fmt::Display;

pub mod error;
mod slice;
pub mod str;


/// The public API of a parser: the result of the parse, and the set of errors
/// that were recovered from.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParseResults<T, L> {
    /// The result of the parse.
    pub result: ParseResult<T, L>,

    /// The set of errors we recovered from during the parse.
    pub errors: Vec<error::ParseErr<L>>,
}

impl<T, L> ParseResults<T, L> {
    /// Convert into a simple `Result`, ignoring any recovered errors.
    pub fn into_result(self) -> Result<T, error::ParseError<L>> {
        match self.result {
            ParseResult::Matched(v) => Ok(v),
            ParseResult::Failed(e) => Err(e),
            ParseResult::Error(e) => Err(
                error::ParseError {
                    location: e.location,
                    expected: error::ExpectedSet::singleton(e.error)
                }
            ),
        }
    }
}

impl<T, L: Display> ParseResults<T, L> {
    /// Return the contained match, or panic on failure or error.
    pub fn unwrap(self) -> T {
        match self.result {
            ParseResult::Matched(v) => v,
            ParseResult::Failed(e) => panic!("parse failed: {}", e),
            ParseResult::Error(e) => panic!("parse {}", e),
        }
    }
}

/// The public result of a parser.
/// A parse may succeed, fail, or raise a named error.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ParseResult<T, L> {
    /// Success
    Matched(T),

    /// Failure
    Failed(error::ParseError<L>),

    /// Labelled error at location
    Error(error::ParseErr<L>),
}

/// The result type used internally in the parser.
///
/// You'll only need this if implementing the `Parse*` traits for a custom input
/// type.
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum RuleResult<T> {
    /// Success, with final location
    Matched(usize, T),

    /// Failure (furthest failure location is not yet known)
    Failed,

    /// Labelled error at location
    Error(error::ParseErr<usize>),
}

/// A type that can be used as input to a parser.
pub trait Parse {
    type PositionRepr: Display;
    fn start<'input>(&'input self) -> usize;
    fn is_eof<'input>(&'input self, p: usize) -> bool;
    fn position_repr<'input>(&'input self, p: usize) -> Self::PositionRepr;
}

/// A parser input type supporting the `[...]` syntax.
pub trait ParseElem: Parse {
    /// Type of a single atomic element of the input, for example a character or token
    type Element;

    /// Get the element at `pos`, or `Failed` if past end of input.
    fn parse_elem(&self, pos: usize) -> RuleResult<Self::Element>;
}

/// A parser input type supporting the `"literal"` syntax.
pub trait ParseLiteral: Parse {
    /// Attempt to match the `literal` string at `pos`, returning whether it
    /// matched or failed.
    fn parse_string_literal(&self, pos: usize, literal: &str) -> RuleResult<()>;
}

/// A parser input type supporting the `$()` syntax.
pub trait ParseSlice<'input>: Parse {
    /// Type of a slice of the input.
    type Slice;

    /// Get a slice of input.
    fn parse_slice(&'input self, p1: usize, p2: usize) -> Self::Slice;
}
