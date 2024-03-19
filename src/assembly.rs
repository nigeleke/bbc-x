use crate::ast::{Identifier, SourceProgramWord};

use std::collections::HashMap;

pub(crate) type SymbolTable = HashMap<Identifier, MemoryAddress>;

pub(crate) type Code = Vec<WordContent>;

pub(crate) type MemoryAddress = usize;

pub(crate) type WordContent = SourceProgramWord;

#[derive(Clone, Default)]
pub(crate) struct Assembly {
    symbol_table: SymbolTable,
    code: Code,
}

impl Assembly {
    pub(crate) fn new(symbol_table: &SymbolTable, code: &Code) -> Self {
        let symbol_table = symbol_table.clone();
        let code = code.clone();
        Self { symbol_table, code }
    }

    pub(crate) fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    #[cfg(test)]
    pub(crate) fn symbol(&self, identifier: &str) -> Option<MemoryAddress> {
        let identifier: Identifier = identifier.into();
        self.symbol_table.get(&identifier).map(|&u| u)
    }

    #[cfg(test)]
    pub(crate) fn content(&self, address: MemoryAddress) -> Option<SourceProgramWord> {
        self.code.get(address).map(|w| w.clone())
    }
}
