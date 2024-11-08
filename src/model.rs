/// The language model enables concrete implementations of the model to parse, assemble 
/// and execute source code.
/// 
use crate::result::*;

use std::path::Path;

pub trait LanguageModel {
    fn assemble(&self, path: &Path) -> Result<()>;
    fn run(&self, path: &Path) -> Result<()>;
    fn list(&self, path: &Path) -> Result<()>;
}
