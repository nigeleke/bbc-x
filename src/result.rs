use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Error)]
pub enum Error {
    #[error("cannot read file: {0} {1}")]
    CannotReadFile(String, String),

    #[error("failed to parse {0}")]
    FailedToParse(String),

    #[error("failed to assemble {0}")]
    FailedToAssemble(String),

    #[error("failed to run {0}")]
    FailedToRun(String),

    #[error("multiple build errors")]
    BuildErrors(Vec<Error>),

    #[error("cannot write file: {0} {1}")]
    CannotToWriteFile(String, String),
}

pub type Result<T> = std::result::Result<T, Error>;
