use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Error)]
pub enum Error {
    #[error("cannot convert word to intruction: {0}")]
    CannotConvertWordToInstruction(String),

    #[error("cannot determine operand: reason: {0}")]
    CannotDetermineOperand(String),

    #[error("failed to create execution context: reason {0}")]
    FailedToCreateExecutionContext(String),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
