pub(crate) mod args;
pub(crate) mod assembler;
pub(crate) mod assembly;
pub(crate) mod ast;
pub(crate) mod bbc_x;
pub(crate) mod grammar;
pub(crate) mod list_writer;
pub(crate) mod parser;
pub(crate) mod result;

use crate::args::Args;
use crate::result::Result;

#[cfg(not(tarpaulin_include))]
fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    match bbc_x(args) {
        Ok(_) => {},
        Err(e) => println!("Error: {:?}", e),
    }
}

fn bbc_x(args: Vec<String>) -> Result<()> {
    let args = Args::from(args);
    bbc_x::bbc_x(&args)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn invoke_core() {
        let args = vec!["bbc-x", "./examples/test/nthg.bbc"].into_iter().map(|s| s.to_string()).collect();
        let result = bbc_x(args);
        assert!(result.is_ok())
    }
}
