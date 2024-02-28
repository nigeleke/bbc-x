#[derive(Debug, PartialEq)]
pub(crate) enum Error {
    InvalidSource(String)
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
