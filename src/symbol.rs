use std::borrow::Cow;
use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use crate::de::FromValue;
use crate::ser::ToValue;

/// A Ruby symbol.
///
/// # Examples
///
/// ```rust
/// # use mruby::symbol::Symbol;
/// let symbol = Symbol::new("hello_world");
/// assert_eq!(symbol.to_string(), ":hello_world");
/// ```
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Symbol(String);

impl Symbol {
    pub fn new<T: Display>(name: T) -> Self {
        Symbol(name.to_string())
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for Symbol {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        let Symbol(ref sym) = *self;

        let is_first_alpha = sym.starts_with(|c: char| c.is_ascii_alphabetic());
        let is_rest_alphanum = sym.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

        if is_first_alpha && is_rest_alphanum {
            write!(fmt, ":{}", sym)
        } else {
            write!(fmt, ":\"{}\"", sym.escape_default())
        }
    }
}

impl Debug for Symbol {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{}({})", stringify!(Symbol), self.to_string())
    }
}

impl FromStr for Symbol {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Symbol::new(s))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InvalidSymbolError(String);

impl InvalidSymbolError {
    pub fn new<T: Display>(value: T) -> Self {
        InvalidSymbolError(value.to_string())
    }
}

impl Display for InvalidSymbolError {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "`{}` is not a recognized symbol", self.0)
    }
}

impl Error for InvalidSymbolError {}

pub trait FromSymbol<'a>: FromValue {
    fn from_name(s: &'a str) -> Result<Self, InvalidSymbolError>;
}

impl<'a> FromSymbol<'a> for Symbol {
    fn from_name(s: &'a str) -> Result<Self, InvalidSymbolError> {
        Ok(Symbol::new(s))
    }
}

impl<'a> FromSymbol<'a> for &'a str {
    fn from_name(s: &'a str) -> Result<Self, InvalidSymbolError> {
        Ok(s)
    }
}

impl<'a> FromSymbol<'a> for String {
    fn from_name(s: &'a str) -> Result<Self, InvalidSymbolError> {
        Ok(s.to_string())
    }
}

impl<'a, T> FromSymbol<'a> for Cow<'a, T>
where
    T: FromSymbol<'a> + ToOwned + 'a,
    Self: From<T> + FromValue,
{
    fn from_name(s: &'a str) -> Result<Self, InvalidSymbolError> {
        T::from_name(s).map(Cow::from)
    }
}

pub trait ToSymbol: ToValue {
    fn as_str(&self) -> &str;

    fn to_symbol(&self) -> Symbol {
        Symbol::new(self.as_str())
    }
}

impl ToSymbol for Symbol {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl ToSymbol for str {
    fn as_str(&self) -> &str {
        self
    }
}

impl ToSymbol for String {
    fn as_str(&self) -> &str {
        &self
    }
}

impl<'a, T> ToSymbol for &'a T
where
    T: ToSymbol + ?Sized,
{
    fn as_str(&self) -> &str {
        (*self).as_str()
    }
}

impl<'a, T> ToSymbol for Cow<'a, T>
where
    T: ToOwned + ToSymbol + 'a,
{
    fn as_str(&self) -> &str {
        self.as_ref().as_str()
    }
}
