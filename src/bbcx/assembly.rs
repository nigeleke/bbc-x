use super::ast::{Label, Location as AstLocation, SourceProgramWord as AstSourceProgramWord};

use std::collections::HashMap;

pub type Location = AstLocation;
pub type Content = AstSourceProgramWord;

pub type Code = HashMap<Location, Content>;
pub type Symbols = HashMap<String, Location>;

#[derive(Debug)]
pub struct Assembly {
    code: Code,
    symbols: Symbols,
}

impl Assembly {
    pub fn new(code: &Code, symbols: &Symbols) -> Self {
        let code = code.clone();
        let symbols = symbols.clone();
        Self { code, symbols }
    }

    #[cfg(test)]
    pub fn content(&self, location: Location) -> Option<Content> {
        self.code.get(&location).map(|w| w.clone())
    }

    #[cfg(test)]
    pub fn location(&self, label: String) -> Option<Location> {
        self.symbols.get(&label).map(|l| l.clone())
    }
}
