use super::convert::{instruction_to_word, store_operand_to_word};
use super::instruction::*;
use super::result::{Error, Result};
use super::word::*;

use crate::bbcx::ast::{Location as AstLocation, Mnemonic, SourceWord as AstSourceWord};
use crate::bbcx::Assembly;

#[derive(Clone, Debug, PartialEq)]
pub struct State(Vec<Word>);

impl State {
    fn add_source_word(
        mut self,
        location: AstLocation,
        source_word: &AstSourceWord,
        assembly: &Assembly,
    ) -> Result<Self> {
        match source_word {
            AstSourceWord::IWord(i) => {
                self[location] = (*i).try_into()?;
            }
            AstSourceWord::FWord(f) => {
                self[location] = (*f).try_into()?;
            }
            AstSourceWord::PWord(pword) => {
                let operand = pword.store_operand();
                let address = if operand.requires_storage() {
                    let address = self.next_storage_address()?;
                    self[address] = store_operand_to_word(&operand)?;
                    address
                } else {
                    assembly.address_used_by_store_operand(operand).try_into()?
                };

                let instruction = if pword.mnemonic() as usize <= Mnemonic::EXTRA as usize {
                    Builder::new(pword.mnemonic())
                        .with_accumulator(pword.accumulator().as_usize())
                        .with_index_register(pword.index_register())
                        .with_indirect(pword.indirect())
                        .with_page(pword.page())
                        .with_address(address)
                        .build()
                } else {
                    // The EXTRA format is `EXTRA <acc>, <code>`
                    // where
                    //      instruction.accumulator = <acc>
                    //      instruction.address     = <code>
                    //
                    // The other formats are `<MNEMONIC> <acc>`
                    // where
                    //      instruction.accumulator = 1 // (defaulted)
                    //      instruction.address     = <acc>
                    //      and the <code> is derived from the mnemonic.
                    //
                    // This difference is not managed in the grammar, though it should be really.
                    // It gets tweaked here...
                    //
                    // Note: Potential gotcha here when `<MNEMONIC>` is used not specifying the
                    // accumulator in the address part, because we use the address, which defaults
                    // to zero rather then one.
                    //
                    let pseudo_address = pword.mnemonic() as usize - Mnemonic::EXTRA as usize;
                    let acc = if pseudo_address == 0 {
                        pword.accumulator().as_usize()
                    } else {
                        address.memory_index()
                    };
                    Builder::new(Mnemonic::EXTRA)
                        .with_accumulator(acc)
                        .with_address(pseudo_address)
                        .build()
                };
                self[location] = instruction_to_word(&instruction)?;
            }
            AstSourceWord::SWord(s) => self[location] = s.as_str().try_into()?,
        };
        Ok(self)
    }

    fn next_storage_address(&self) -> Result<Address> {
        self.0
            .iter()
            .rposition(Word::is_undefined)
            .and_then(|i| i.try_into().ok())
            .ok_or(Error::OutOfMemory)
    }

    #[cfg(test)]
    pub fn iter(&self) -> std::slice::Iter<'_, Word> {
        self.0.iter()
    }
}

pub const MEMORY_SIZE: usize = 1024;

impl Default for State {
    fn default() -> Self {
        let mut words = vec![Word::new(WordType::IWord, 0)];
        words.extend(vec![Word::default(); MEMORY_SIZE - 1]);
        Self(words)
    }
}

impl TryFrom<Assembly> for State {
    type Error = Error;

    fn try_from(value: Assembly) -> Result<Self> {
        let linked_code = value.linked_code();
        let mut keys = Vec::from_iter(linked_code.keys());
        keys.sort();
        keys.into_iter()
            .try_fold(State::default(), |acc, location| {
                let content = &linked_code[location];
                acc.add_source_word(*location, content, &value)
            })
    }
}

impl std::ops::Index<usize> for State {
    type Output = Word;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for State {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl std::ops::Index<Accumulator> for State {
    type Output = Word;

    fn index(&self, acc: Accumulator) -> &Self::Output {
        &self.0[acc.memory_index()]
    }
}

impl std::ops::Index<Accumulator> for Vec<Word> {
    type Output = Word;

    fn index(&self, acc: Accumulator) -> &Self::Output {
        &self[acc.memory_index()]
    }
}

impl std::ops::IndexMut<Accumulator> for State {
    fn index_mut(&mut self, acc: Accumulator) -> &mut Self::Output {
        &mut self.0[acc.memory_index()]
    }
}

impl std::ops::Index<IndexRegister> for State {
    type Output = Word;

    fn index(&self, index_register: IndexRegister) -> &Self::Output {
        &self.0[index_register.memory_index()]
    }
}

impl std::ops::IndexMut<IndexRegister> for State {
    fn index_mut(&mut self, index_register: IndexRegister) -> &mut Self::Output {
        &mut self.0[index_register.memory_index()]
    }
}

impl std::ops::Index<Address> for State {
    type Output = Word;

    fn index(&self, address: Address) -> &Self::Output {
        &self.0[address.memory_index()]
    }
}

impl std::ops::IndexMut<Address> for State {
    fn index_mut(&mut self, address: Address) -> &mut Self::Output {
        &mut self.0[address.memory_index()]
    }
}
