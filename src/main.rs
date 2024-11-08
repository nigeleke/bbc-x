pub mod args;
pub mod bbc3;
pub mod bbcx;
pub mod core;
pub mod language;
pub mod list_writer;
pub mod model;
pub mod result;

use crate::args::Args;
use crate::core::Core;
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
    Core::build_all(&args)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn invoke_core() {
        let args = vec!["bbc-x", "--lang=bbc3", "./examples/test/bbc3/nthg.bbc"].into_iter().map(|s| s.to_string()).collect();
        let result = bbc_x(args);
        assert!(result.is_ok())
    }
}
