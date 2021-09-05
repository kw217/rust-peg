//! Parse error reporting

use crate::{Parse, RuleResult};
use std::collections::HashSet;
use std::fmt::{self, Debug, Display};

/// A set of literals or names that failed to match
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExpectedSet {
    expected: HashSet<&'static str>,
}

impl ExpectedSet {
    /// Iterator of expected literals
    pub fn tokens<'a>(&'a self) -> impl Iterator<Item = &'static str> + 'a {
        self.expected.iter().map(|x| *x)
    }

    /// Construct a new singleton set.
    pub fn singleton(error: &'static str) -> Self {
        let mut expected = HashSet::new();
        expected.insert(error);
        ExpectedSet { expected }
    }
}

impl Display for ExpectedSet {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.expected.is_empty() {
            write!(fmt, "<unreported>")?;
        } else if self.expected.len() == 1 {
            write!(fmt, "{}", self.expected.iter().next().unwrap())?;
        } else {
            let mut errors = self.tokens().collect::<Vec<_>>();
            errors.sort();
            let mut iter = errors.into_iter();

            write!(fmt, "one of {}", iter.next().unwrap())?;
            for elem in iter {
                write!(fmt, ", {}", elem)?;
            }
        }

        Ok(())
    }
}

/// An error from a parse failure
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ParseError<L> {
    /// The furthest position the parser reached in the input
    pub location: L,

    /// The set of literals that failed to match at that position
    pub expected: ExpectedSet,
}

// TODO Make this a method
pub fn new_parse_error<I: Parse + ?Sized>(input: &I, pos: usize, expected: ExpectedSet) -> ParseError<I::PositionRepr> {
    ParseError {
        location: input.position_repr(pos),
        expected,
    }
}

impl<L: Display> Display for ParseError<L> {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        write!(
            fmt,
            "error at {}: expected {}",
            self.location, self.expected
        )
    }
}

impl<L: Display + Debug> ::std::error::Error for ParseError<L> {
    fn description(&self) -> &str {
        "parse error"
    }
}

#[doc(hidden)]
pub struct ErrorState<I: Parse + ?Sized> {
    /// Furthest failure we've hit so far. Not relevant to errors.
    pub max_err_pos: usize,

    /// Are we inside a lookahead/quiet block? If so, failure/error and recovery rules are disabled.
    /// Non-zero => yes, to support nested blocks.
    pub suppress_fail: usize,

    /// Are we reparsing after an failure? If so, compute and store expected set of all alternative expectations
    /// when we are at offset `max_err_pos`. Not required for errors.
    pub reparsing_on_error: bool,

    /// The set of tokens we expected to find when we hit the failure. Updated when `reparsing_on_error`.
    pub expected: ExpectedSet,

    /// The set of errors we have recovered from so far.
    pub errors: Vec<ParseError<<I as Parse>::PositionRepr>>,
    @@@ just store (str, usize) - this will allow the ErrorState type to be simplified
}

// Not sure why this isn't derivable.
impl<I: Parse + ?Sized> Debug for ErrorState<I> where <I as Parse>::PositionRepr : Debug {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        write!(fmt, "ErrorState {{ max_err_pos: {:?}, suppress_fail: {:?}, reparsing_on_error: {:?}, expected: {:?}, errors: {:?} }}",
            self.max_err_pos, self.suppress_fail, self.reparsing_on_error, self.expected, self.errors)
    }
}

impl<I: Parse + ?Sized> ErrorState<I> {
    pub fn new(initial_pos: usize) -> Self {
        ErrorState {
            max_err_pos: initial_pos,
            suppress_fail: 0,
            reparsing_on_error: false,
            expected: ExpectedSet {
                expected: HashSet::new(),
            },
            errors: vec![],
        }
    }

    pub fn reparse_for_error(&mut self) {
        self.suppress_fail = 0;
        self.reparsing_on_error = true;
    }

    #[inline(never)]
    pub fn mark_failure_slow_path(&mut self, pos: usize, expected: &'static str) {
        if pos == self.max_err_pos {
            self.expected.expected.insert(expected);
        }
    }

    #[inline(always)]
    pub fn mark_failure(&mut self, pos: usize, expected: &'static str) -> RuleResult<()> {
        if self.suppress_fail == 0 {
            if self.reparsing_on_error {
                self.mark_failure_slow_path(pos, expected);
            } else if pos > self.max_err_pos {
                self.max_err_pos = pos;
            }
        }
        RuleResult::Failed
    }

    /// Flag an error.
    #[inline(always)]
    pub fn mark_error(&mut self, input: &I, pos: usize, error: &'static str) {
        if self.suppress_fail == 0 {
            self.errors.push(new_parse_error(input, pos, ExpectedSet::singleton(error)));
        }
    }

    pub fn into_parse_error(self, input: &I) -> ParseError<I::PositionRepr> {
        new_parse_error(input, self.max_err_pos.into(), self.expected)
    }
}
