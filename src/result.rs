#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Error {
     CannotReadFile(String, String),
     FailedToParse(String),
     FailedToAssemble(String),
     FailedToRun(String),
     BuildErrors(Vec<Error>),
     CannotToWriteFile(String, String)
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
