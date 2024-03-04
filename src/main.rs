use bbc_x::*;
use bbc_x::result::Result;

#[cfg(not(tarpaulin_include))]
fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<String>>();
    bbc_x(args)
}
