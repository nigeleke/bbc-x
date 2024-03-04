#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidInput(Vec<Error>),
    InvalidLine(usize, String, String)
}

pub type Result<T> = std::result::Result<T, Error>;
