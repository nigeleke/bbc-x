pub(crate) mod args;
pub(crate) mod assembler;
pub(crate) mod ast;
pub(crate) mod bbc_x;
pub(crate) mod grammar;
pub(crate) mod parser;

pub mod result;

use crate::args::Args;
use crate::result::Result;

pub fn bbc_x(args: Vec<String>) -> Result<()> {
    let args = Args::from(args);
    bbc_x::bbc_x(&args)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn invoke_core() {
        println!("CWD:: {:?}", std::env::current_dir().unwrap().display());
        let args = vec!["bbc-x", "./examples/nthg.bbc"].into_iter().map(|s| s.to_string()).collect();
        let result = bbc_x(args);
        assert!(result.is_ok())
    }
}