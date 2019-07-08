use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    EofWhileParsingString,
    ExpectedString,
    ExpectedTranslatable,
    TrailingCharacters,
    AttributeOrder,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(match self {
            Error::EofWhileParsingString => "EOF while parsing a string",
            Error::ExpectedString => "expected string",
            Error::ExpectedTranslatable => "expected translatable string",
            Error::TrailingCharacters => "extra trailing characters",
            Error::AttributeOrder => "attribute not in order",
        })
    }
}

impl std::error::Error for Error {}
