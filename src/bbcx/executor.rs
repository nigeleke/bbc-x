use super::assembly::Assembly;

use crate::result::Result;

#[derive(Debug, PartialEq)]
pub struct Executor {}

impl Executor {
    pub fn execute(_assembly: Assembly) -> Result<()> {
        unimplemented!()
    }
}

// #[cfg(test)]
// mod test {
//     #[test]
//     fn
// }
