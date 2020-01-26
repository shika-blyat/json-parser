use std::fmt;

mod combinator;
mod error;

pub use crate::parsec::combinator::{character, multi, whitespace};
pub use crate::parsec::error::ParserError;

pub trait Parser<'a, T> =
    FnMut(Remaining<'a>) -> Result<(Remaining<'a>, T), (Remaining<'a>, ParserError)>;

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

pub trait IntoRem<'a>: Sized {
    fn into_rem(self) -> Remaining<'a>;
}

impl<'a> IntoRem<'a> for &'a str {
    fn into_rem(self) -> Remaining<'a> {
        Remaining::new(self, 0)
    }
}

impl<'a> IntoRem<'a> for Remaining<'a> {
    fn into_rem(self) -> Remaining<'a> {
        self
    }
}
