use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::Utf8Error;

use mruby_sys::mrb_int;

use crate::symbol::InvalidSymbolError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CastError {
    InvalidSymbol(InvalidSymbolError),
    Length(usize, usize),
    NumericConversion(mrb_int, &'static str),
    UnexpectedType(String),
    Utf8(Utf8Error),
}

impl CastError {
    pub const fn length(expected: usize, actual: usize) -> Self {
        CastError::Length(expected, actual)
    }

    pub const fn numeric_conversion(value: mrb_int, rust_type: &'static str) -> Self {
        CastError::NumericConversion(value, rust_type)
    }

    pub fn unexpected_type<C: Display>(context_msg: C) -> Self {
        CastError::UnexpectedType(context_msg.to_string())
    }
}

impl Display for CastError {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            CastError::InvalidSymbol(ref err) => err.fmt(fmt),
            CastError::Length(ref expected, ref found) => write!(
                fmt,
                "incorrect array or tuple length: expected {}, found {}",
                expected, found
            ),
            CastError::NumericConversion(ref num, ref ty) => write!(
                fmt,
                "numeric conversion failed: '{}' cannot be cast as '{}'",
                num, ty
            ),
            CastError::UnexpectedType(ref msg) => write!(fmt, "unexpected Rust type: {}", msg),
            CastError::Utf8(ref err) => err.fmt(fmt),
        }
    }
}

impl Error for CastError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            CastError::InvalidSymbol(ref err) => Some(err),
            CastError::Utf8(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<InvalidSymbolError> for CastError {
    fn from(err: InvalidSymbolError) -> Self {
        CastError::InvalidSymbol(err)
    }
}

impl From<Utf8Error> for CastError {
    fn from(err: Utf8Error) -> Self {
        CastError::Utf8(err)
    }
}
