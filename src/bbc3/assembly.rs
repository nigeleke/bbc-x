use super::ast::{Location as AstLocation, SourceProgramWord as AstSourceProgramWord};

use std::collections::HashMap;

pub type Location = AstLocation;
pub type Content = AstSourceProgramWord;

pub type Code = HashMap<Location, Content>;

pub struct Assembly {
    _code: Code,
}

impl Assembly {
    pub fn new(code: &Code) -> Self {
        let code = code.clone();
        Self { _code: code }
    }

    #[cfg(test)]
    pub fn content(&self, location: Location) -> Option<Content> {
        self._code.get(&location).map(|w| w.clone())
    }
}
