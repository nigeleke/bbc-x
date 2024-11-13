use super::ast::{
    Address as AstAddress, Identifier, Location as AstLocation,
    SimpleAddressOperand as AstSimpleAddressOperand, SourceWord as AstSourceProgramWord,
    StoreOperand as AstStoreOperand,
};

use std::collections::HashMap;

pub type Location = AstLocation;
pub type Content = AstSourceProgramWord;

pub type Code = HashMap<Location, Content>;
pub type Symbols = HashMap<Identifier, Location>;

#[derive(Clone, Debug, PartialEq)]
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
        self.code.get(&location).cloned()
    }

    pub fn code_iter(&self) -> impl Iterator<Item = (&Location, &Content)> {
        let mut keys = Vec::from_iter(self.code.keys());
        keys.sort();
        keys.into_iter().map(|k| (k, &self.code[k]))
    }

    pub fn location(&self, label: &str) -> Option<Location> {
        self.symbols.get(label).copied()
    }

    pub fn allocate_storage_locations(mut self) -> Self {
        let undefined_symbols = self.undefined_symbols();
        let mut store_location: usize = 1024;
        undefined_symbols.into_iter().for_each(|identifier| {
            store_location -= 1;
            self.symbols.insert(identifier, store_location);
        });
        self
    }

    fn undefined_symbols(&self) -> Vec<Identifier> {
        self.code_iter()
            .filter_map(|(_, content)| match content {
                AstSourceProgramWord::PWord(pword) => Some(pword),
                _ => None,
            })
            .filter_map(|pword| match pword.store_operand() {
                AstStoreOperand::None => None,
                AstStoreOperand::ConstOperand(_) => None,
                AstStoreOperand::AddressOperand(operand) => Some(operand.address()),
            })
            .map(|address| match address {
                AstSimpleAddressOperand::DirectAddress(address) => address,
                AstSimpleAddressOperand::IndirectAddress(address) => address,
            })
            .filter_map(|address| match address {
                AstAddress::Identifier(identifer) => Some(identifer),
                AstAddress::NumericAddress(_) => None,
            })
            .filter_map(|identifier| match self.location(&identifier) {
                Some(_) => None,
                None => Some(identifier),
            })
            .collect::<Vec<_>>()
    }

    pub fn first_pword_location(&self) -> Option<Location> {
        self.code_iter()
            .find_map(|(location, content)| match content {
                AstSourceProgramWord::PWord(_) => Some(*location),
                _ => None,
            })
    }
}
