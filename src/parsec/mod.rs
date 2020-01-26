use std::fmt;

mod combinator;
mod error;

pub use crate::parsec::combinator::{character, multi, whitespace};
pub use crate::parsec::error::ParserError;

#[derive(Debug)]
pub enum JsonError<'a> {
    Savable(Remaining<'a>),
    Failure(Remaining<'a>, ParserError),
    Unsavable(usize, ParserError),
}
impl<'a> JsonError<'a> {
    pub fn rem(&self) -> Remaining<'a> {
        match self {
            Self::Failure(rem, _) => *rem,
            Self::Savable(rem) => *rem,
            Self::Unsavable(_, _) => panic!(
                "Internal parser error, `Unexpected JsonError.rem() call on an Unsavable variant"
            ),
        }
    }
}
pub trait Parser<'a, T> = FnMut(Remaining<'a>) -> Result<(Remaining<'a>, T), JsonError>;

#[derive(Debug, Clone, Copy)]
pub struct Remaining<'a> {
    pub pos: usize,
    pub rem: &'a str,
}

impl<'a> Remaining<'a> {
    pub fn new(rem: &'a str, pos: usize) -> Self {
        Self { rem, pos }
    }
    pub fn rem_len(&self) -> usize {
        self.rem.len()
    }
}
impl<'a> fmt::Display for Remaining<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rem)
    }
}
