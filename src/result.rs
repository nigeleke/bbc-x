use crate::parser::ParsedLine;

#[derive(Debug, PartialEq)]
pub enum Error {
     CannotReadFile(String),
     InvalidInput(Vec<ParsedLine>),
     InvalidLine(String, String)
}

pub type Result<T> = std::result::Result<T, Error>;

// // impl<T: PartialEq> PartialEq for Result<T> {
// //     fn eq(&self, other: &Self) -> bool {
// //         match (self, other) {
// //             (Ok(value1), Ok(value2)) => value1 == value2,
// //             (Err(err1), Err(err2)) => err1 == err2,
// //             (_, _) => false,
// //         }
// //     }
// // }
