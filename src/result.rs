#[derive(Debug, PartialEq)]
pub(crate) enum Error {
    InvalidInput(Vec<Error>),
    InvalidLine(usize, String, String)
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
