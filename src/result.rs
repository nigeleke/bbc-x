use crate::parser::ParsedLine;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SymbolError {
     Undefined(String),
     Duplicated(String),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Error {
     CannotReadFile(String),
     InvalidInput(Vec<ParsedLine>),
     InvalidLine(String, String),
     UnresolvedSymbols(Vec<SymbolError>),
     BuildErrors(Vec<Error>),
     UnableToWriteFile(String)
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
