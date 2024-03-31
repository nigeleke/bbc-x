use super::ast::{Location as AstLocation, SourceProgramWord as AstSourceProgramWord};

use std::collections::HashMap;

pub(crate) type Location = AstLocation;
pub(crate) type Content = AstSourceProgramWord;

pub(crate) type Code = HashMap<Location, Content>;

pub(crate) struct Assembly {
    code: Code,
}

impl Assembly {
    pub(crate) fn new(code: &Code) -> Self {
        let code = code.clone();
        Self { code }
    }

    #[cfg(test)]
    pub(crate) fn content(&self, location: Location) -> Option<Content> {
        self.code.get(&location).map(|w| w.clone())
    }
}
