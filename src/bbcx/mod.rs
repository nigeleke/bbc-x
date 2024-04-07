use crate::args::Args;
use crate::model::*;

use crate::result::Result;

use std::path::Path ;

// #[derive(Clone)]
pub(crate) struct BbcX {
    _args: Args,
}

impl BbcX {
    pub(crate) fn new(args: &Args) -> BbcX {
        let args = args.clone();
        Self { _args: args }
    }
}

impl LanguageModel for BbcX {
    fn assemble(&self, _path: &Path) -> Result<()> { unimplemented!() }
    fn run(&self, _path: &Path) -> Result<()> { unimplemented!() }
    fn list(&self, _path: &Path) -> Result<()> { unimplemented!() }
}

#[cfg(test)]
mod test {
    #[test]
    fn will_assemble() {
        
    }
    #[test]
    fn will_not_run() {
        
    }
    #[test]
    fn will_list() {
        
    }
}