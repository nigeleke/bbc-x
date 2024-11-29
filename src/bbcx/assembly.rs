use super::ast::{
    Address as AstAddress, AddressOperand, Identifier, Location as AstLocation, PWord as AstPWord,
    SimpleAddressOperand as AstSimpleAddressOperand, SourceWord as AstSourceWord,
    StoreOperand as AstStoreOperand,
};

use std::collections::HashMap;

pub type Location = AstLocation;
pub type Content = AstSourceWord;

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

    fn code_iter(&self) -> impl Iterator<Item = (&Location, &Content)> {
        let mut keys = Vec::from_iter(self.code.keys());
        keys.sort();
        keys.into_iter()
            .map(|location| (location, &self.code[location]))
    }

    pub fn linked_code(&self) -> Code {
        let mut keys = Vec::from_iter(self.code.keys());
        keys.sort();
        keys.into_iter()
            .map(|location| (*location, self.linked_content(&self.code[location])))
            .collect::<Code>()
    }

    fn linked_content(&self, content: &Content) -> Content {
        match content {
            Content::PWord(pword) => Content::PWord(self.linked_pword(pword)),
            other => other.clone(),
        }
    }

    fn linked_pword(&self, pword: &AstPWord) -> AstPWord {
        AstPWord::new(
            pword.mnemonic(),
            pword.accumulator(),
            self.linked_storage_operand(pword.store_operand()),
        )
    }

    fn linked_storage_operand(&self, operand: AstStoreOperand) -> AstStoreOperand {
        match operand {
            AstStoreOperand::AddressOperand(address_operand) => {
                AstStoreOperand::AddressOperand(self.linked_address_operand(address_operand))
            }
            other => other,
        }
    }

    pub fn address_used_by_store_operand(&self, operand: AstStoreOperand) -> Location {
        match operand {
            AstStoreOperand::None => 0,
            AstStoreOperand::AddressOperand(address_operand) => {
                self.address_used_by_address_operand(address_operand)
            }
            _ => unreachable!(),
        }
    }

    fn linked_address_operand(&self, operand: AddressOperand) -> AddressOperand {
        AddressOperand::new(
            self.linked_simple_address_operand(operand.address()),
            operand.index(),
        )
    }

    fn address_used_by_address_operand(&self, operand: AddressOperand) -> Location {
        self.address_used_by_simple_address_operand(operand.address())
    }

    fn linked_simple_address_operand(
        &self,
        operand: AstSimpleAddressOperand,
    ) -> AstSimpleAddressOperand {
        match operand {
            AstSimpleAddressOperand::DirectAddress(address) => {
                AstSimpleAddressOperand::DirectAddress(self.linked_address(address))
            }
            AstSimpleAddressOperand::IndirectAddress(address) => {
                AstSimpleAddressOperand::IndirectAddress(self.linked_address(address))
            }
        }
    }

    fn address_used_by_simple_address_operand(&self, operand: AstSimpleAddressOperand) -> Location {
        match operand {
            AstSimpleAddressOperand::DirectAddress(address)
            | AstSimpleAddressOperand::IndirectAddress(address) => self.address_used_by(address),
        }
    }

    fn linked_address(&self, address: AstAddress) -> AstAddress {
        match address {
            AstAddress::Identifier(id) => AstAddress::NumericAddress(self.symbols[&id]),
            other => other,
        }
    }

    fn address_used_by(&self, address: AstAddress) -> Location {
        match address {
            AstAddress::NumericAddress(a) => a,
            _ => unreachable!(),
        }
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
                AstSourceWord::PWord(pword) => Some(pword),
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
                AstSourceWord::PWord(_) => Some(*location),
                _ => None,
            })
    }
}
