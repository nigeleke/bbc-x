use crate::args::Args;
use crate::result::Result;

pub fn bbc_x(args: &Args) -> Result<()> { 
    println!("Args: {:?}", args);
    Ok(())
}
