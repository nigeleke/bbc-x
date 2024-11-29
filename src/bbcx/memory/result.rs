use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Error)]
pub enum Error {
    #[error("cannot create IWord from {0}")]
    InvalidIWordValue(i64),

    #[error("cannot create FWord from {0}")]
    InvalidFWordValue(f64),

    #[error("cannot create SWord from {0}")]
    InvalidSWordValue(String),

    #[error("invalid accumulator {0}")]
    InvalidAccumulator(usize),

    #[error("invalid address {0}")]
    InvalidAddress(usize),

    #[error("invalid page {0}")]
    InvalidPage(usize),

    // TODO: Get rid of this...
    #[error("cannot create Word from {0}")]
    CannotCreateWordFromStoreOperand(String),

    #[error("error converting word into target value {0}")]
    CannotConvertFromWord(String),

    #[error("cannot allocate memory")]
    OutOfMemory,
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
