use super::instruction::*;
use super::result::{Error, Result};
use super::word::{bits, Word, WordType};

use crate::bbcx::ast::{ConstOperand, StoreOperand};

use num_enum::TryFromPrimitive;

pub fn word_to_instruction(word: &Word) -> Result<Instruction> {
    let function = Function::try_from_primitive(word.pword_function_bits() as u32)
        .map_err(|err| Error::CannotConvertFromWord(err.to_string()))?;
    let instruction = Builder::new(function)
        .with_accumulator(word.pword_accumulator_bits() as usize)
        .with_index_register(word.pword_index_register_bits() as usize)
        .with_indirect(word.pword_indirect_bits() != 0)
        .with_page(word.pword_page_bits() as usize)
        .with_address(word.pword_address_bits() as usize)
        .build();
    Ok(instruction)
}

pub fn instruction_to_word(instruction: &Instruction) -> Result<Word> {
    let function: u32 = instruction.function().into();
    let acc = instruction.accumulator().bits();
    let index_register = instruction.index_register().bits();
    let indirect = instruction.indirect().bits();
    let page = instruction.page().bits();
    let address = instruction.address().bits();

    let raw = bits::set(function, Word::PWORD_FUNCTION_MASK)
        | bits::set(acc, Word::PWORD_ACCUMULATOR_MASK)
        | bits::set(index_register, Word::PWORD_INDEX_REGISTER_MASK)
        | bits::set(indirect, Word::PWORD_INDIRECT_MASK)
        | bits::set(page, Word::PWORD_PAGE_MASK)
        | bits::set(address, Word::PWORD_ADDRESS_MASK);

    Ok(Word::new(WordType::PWord, raw))
}

pub fn store_operand_to_word(operand: &StoreOperand) -> Result<Word> {
    match operand {
        StoreOperand::ConstOperand(operand) => match operand {
            ConstOperand::SignedIWord(i) => (*i).try_into(),
            ConstOperand::SignedFWord(f) => (*f).try_into(),
            ConstOperand::SWord(s) => s.as_str().try_into(),
        },
        _ => Err(Error::CannotCreateWordFromStoreOperand(operand.to_string())),
    }
}
