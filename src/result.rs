use crate::parser::ParsedLine;

#[derive(Debug, PartialEq)]
pub(crate) enum Error {
     CannotReadFile(String),
     InvalidInput(Vec<ParsedLine>),
     InvalidLine(String, String),
     UndefinedSymbols(Vec<String>),
     DuplicatedSymbols(Vec<String>),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
