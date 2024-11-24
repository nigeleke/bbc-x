mod instruction;
mod word;

pub use super::memory::instruction::Instruction;
pub use super::memory::word::{Word, OVERFLOW_MASK, WORD_MASK, WORD_SIZE};

use crate::bbcx::assembly::Assembly;
use crate::bbcx::ast::{
    FloatType as AstFloatType, IndexRegister as AstIndexRegister, IntType as AstIntType,
    Location as AstLocation, Mnemonic as AstMnemonic, SourceWord as AstSourceWord,
};

pub type Function = AstMnemonic;
pub type IndexRegister = AstIndexRegister;
pub type IntType = AstIntType;
pub type FloatType = AstFloatType;

pub type Offset = usize;

#[derive(Clone, Debug, PartialEq)]
pub struct Memory(Vec<Word>);

impl Memory {
    fn add_source_word(
        mut self,
        location: AstLocation,
        source_word: &AstSourceWord,
        assembly: &Assembly,
    ) -> Self {
        match source_word {
            AstSourceWord::IWord(i) => {
                self[location] = Word::IWord(*i);
            }
            AstSourceWord::FWord(f) => {
                self[location] = Word::FWord(*f);
            }
            AstSourceWord::PWord(pword) => {
                let operand = pword.store_operand();
                let address = if operand.requires_storage() {
                    let address = self.next_storage_address();
                    self[address] = Word::from(operand);
                    address
                } else {
                    assembly.address_used_by_store_operand(operand)
                };

                let instruction = Instruction::new(pword.mnemonic())
                    .with_accumulator(pword.accumulator().as_usize())
                    .with_index_register(pword.index_register())
                    .with_indirect(pword.indirect())
                    .with_page(pword.page())
                    .with_address(address);
                self[location] = Word::PWord(instruction);
            }
            AstSourceWord::SWord(s) => self[location] = s.as_str().into(),
        };
        self
    }

    fn next_storage_address(&self) -> Offset {
        self.0
            .iter()
            .rposition(|content| *content == Word::Undefined)
            .unwrap()
    }
}

pub const MEMORY_SIZE: Offset = 128; // TODO Change to 1024

impl Default for Memory {
    fn default() -> Self {
        Self(vec![Word::default(); MEMORY_SIZE])
    }
}

impl From<Assembly> for Memory {
    fn from(value: Assembly) -> Self {
        let linked_code = value.linked_code();
        let mut keys = Vec::from_iter(linked_code.keys());
        keys.sort();
        keys.into_iter().fold(Memory::default(), |acc, location| {
            let content = &linked_code[location];
            acc.add_source_word(*location, content, &value)
        })
    }
}

impl std::ops::Index<usize> for Memory {
    type Output = Word;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn can_create_default_word() {
        assert_eq!(Word::default(), Word::Undefined)
    }

    #[test]
    fn can_convert_between_iword_and_raw_bits() {
        let expected = Word::IWord(42);
        let raw = expected.raw_bits();
        let actual = Word::iword_from(raw);
        assert_eq!(actual, expected);
    }

    #[test]
    fn can_convert_between_fword_and_raw_bits() {
        let expected = Word::FWord(42.0);
        let raw = expected.raw_bits();
        let actual = Word::fword_from(raw);
        assert_eq!(actual, expected);
    }

    #[test]
    fn can_convert_between_sword_and_raw_bits() {
        let expected: Word = "ABCD".into();
        let raw = expected.raw_bits();
        let actual = Word::sword_from(raw);
        assert_eq!(actual, expected);
    }
}
