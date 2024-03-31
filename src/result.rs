#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Error {
     CannotReadFile(String, String),
     FailedToParse(String),
     FailedToAssemble(String),
     FailedToRun(String),
     BuildErrors(Vec<Error>),
     CannotToWriteFile(String, String)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
