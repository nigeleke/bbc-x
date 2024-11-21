use crate::main;

use super::assembly::Assembly;
use super::ast::{
    ConstOperand as AstConstOperand, FloatType as AstFloatType, IndexRegister as AstIndexRegister,
    IntType as AstIntType, Location as AstLocation, Mnemonic as AstMnemonic,
    SourceWord as AstSourceWord, StoreOperand as AstStoreOperand,
};
use super::charset::CharSet;

pub type Function = AstMnemonic;
pub type IndexRegister = AstIndexRegister;
pub type IntType = AstIntType;
pub type FloatType = AstFloatType;

pub type Offset = usize;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Instruction {
    function: Function,
    accumulator: Offset,
    index_register: IndexRegister,
    indirect: bool,
    page: usize,
    address: Offset,
}

impl Instruction {
    pub fn new(function: Function) -> Self {
        Self {
            function,
            ..Default::default()
        }
    }

    pub fn with_accumulator(mut self, accumulator: Offset) -> Self {
        self.accumulator = accumulator;
        self
    }

    pub fn with_index_register(mut self, index_register: IndexRegister) -> Self {
        self.index_register = index_register;
        self
    }

    pub fn with_indirect(mut self, indirect: bool) -> Self {
        self.indirect = indirect;
        self
    }

    pub fn with_page(mut self, page: usize) -> Self {
        self.page = page;
        self
    }

    pub fn with_address(mut self, address: Offset) -> Self {
        self.address = address;
        self
    }

    pub fn function(&self) -> Function {
        self.function
    }

    pub fn accumulator(&self) -> Offset {
        self.accumulator
    }

    pub fn index_register(&self) -> IndexRegister {
        self.index_register
    }

    pub fn indirect(&self) -> bool {
        self.indirect
    }

    pub fn address(&self) -> Offset {
        self.address
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Word {
    #[default]
    Undefined,
    IWord(IntType),
    FWord(FloatType),
    SWord([u8; 4]),
    PWord(Instruction),
}

pub type RawBits = u32;

const WORD_MASK: RawBits = 0o7777_7777;

const PWORD_FUNCTION_MASK: RawBits = 0o7700_0000;
const PWORD_ACCUMULATOR_MASK: RawBits = 0o0070_0000;
const PWORD_INDEX_REGISTER_MASK: RawBits = 0o0007_0000;
const PWORD_INDIRECT_MASK: RawBits = 0o0000_4000;
const PWORD_PAGE_MASK: RawBits = 0o0000_2000;
const PWORD_ADDRESS_MASK: RawBits = 0o0000_1777;

const IWORD_SIGN_MASK: RawBits = 0o4000_0000;
const IWORD_INTEGER_MASK: RawBits = 0o3777_7777;

const FWORD_SIGN_MASK: RawBits = 0o4000_0000;
const FWORD_EXPONENT_MASK: RawBits = 0o3760_0000;
const FWORD_MANTISSA_MASK: RawBits = 0o0017_7777;

impl Word {
    fn iword_from(raw: RawBits) -> Self {
        let i64_value = ((raw as i64) << 32) >> 32;
        Word::IWord(i64_value)
    }

    fn fword_from(raw: RawBits) -> Self {
        let sign = if (raw & FWORD_SIGN_MASK) != 0 {
            -1.0
        } else {
            1.0
        };

        let raw_exponent = (raw & FWORD_EXPONENT_MASK) >> 16;
        let exponent = (raw_exponent as i32 - 63) as i32;

        let raw_mantissa = raw & FWORD_MANTISSA_MASK;
        let mantissa = (raw_mantissa as f64) / ((1 << 17) as f64);

        Word::FWord(sign * (1.0 + mantissa) * (2.0_f64).powi(exponent))
    }

    fn sword_from(raw: RawBits) -> Self {
        Word::SWord([
            ((raw & 0o77000000) >> 18) as u8,
            ((raw & 0o00770000) >> 12) as u8,
            ((raw & 0o00007700) >> 6) as u8,
            (raw & 0o00000077) as u8,
        ])
    }

    fn pword_from(_raw: RawBits) -> Self {
        unimplemented!()
    }

    fn same_type_from(&self, raw: RawBits) -> Self {
        match self {
            Word::Undefined => *self,
            Word::IWord(_) => Word::iword_from(raw),
            Word::FWord(_) => Word::fword_from(raw),
            Word::SWord(_) => Word::sword_from(raw),
            Word::PWord(_) => Word::pword_from(raw),
        }
    }

    pub fn raw_bits(&self) -> RawBits {
        match &self {
            Word::Undefined => 0,
            Word::IWord(i) => *i as RawBits & WORD_MASK,
            Word::FWord(f) => {
                /*
                // Extract the components from the 24-bit raw representation (we use u32 for convenience)
                let sign = (raw >> 23) & 0x1; // Extract the sign bit (1 bit)
                let exponent = ((raw >> 16) & 0x7F) as i8; // Extract the exponent (7 bits), treat as signed
                let mantissa = (raw & 0xFFFF) as u16; // Extract the mantissa (16 bits)

                // Calculate the exponent (subtract bias of 63)
                let exponent = exponent - 63;

                // Convert mantissa to fractional part
                let mantissa_fraction = mantissa as f64 / (1 << 16) as f64;

                // Recreate the floating-point value
                let value = (-1.0f64).powi(sign as i32) * 2f64.powi(exponent as i32) * (1.0 + mantissa_fraction);

                value */
                let dissect_f64 = |value: f64| {
                    let bits = value.to_bits();
                    let sign = (bits >> 63) & 1;
                    let exponent = ((bits >> 52) & 0o3777) - 1023;
                    let mantissa = bits & 0o17_7777_7777_7777_7777;
                    (sign, exponent, mantissa)
                };
                let (sign, exponent, mantissa) = dissect_f64(*f);

                ((sign << 23) | ((exponent + 63) << 16) | (mantissa >> 35)) as RawBits
            }
            Word::SWord(s) => {
                ((s[0] as RawBits) << 18)
                    | ((s[1] as RawBits) << 12)
                    | ((s[2] as RawBits) << 6)
                    | s[3] as RawBits
            }
            Word::PWord(instruction) => {
                let function_code = instruction.function as RawBits;
                let accumulator = instruction.accumulator as RawBits;
                let index_register = instruction.index_register as RawBits;
                let indirect = instruction.indirect as RawBits;
                let page = instruction.page as RawBits;
                let address = instruction.address as RawBits;
                ((function_code << 18)
                    | (accumulator << 15)
                    | (index_register << 12)
                    | (indirect << 11)
                    | (page << 10)
                    | address) as RawBits
            }
        }
    }
}

impl From<&str> for Word {
    fn from(value: &str) -> Self {
        assert!(
            value.len() <= 4,
            "String too long. Expected length <= 4 bytes"
        );

        let mut buffer = [0u8; 4];
        for (i, c) in value.as_bytes().iter().enumerate() {
            let bits = CharSet::char_to_bits(*c).expect("Invalid character for conversion"); // Handle error better if needed
            buffer[i] = bits as u8;
        }
        Word::SWord(buffer)
    }
}

impl From<AstStoreOperand> for Word {
    fn from(value: AstStoreOperand) -> Self {
        match value {
            AstStoreOperand::ConstOperand(operand) => match operand {
                AstConstOperand::SignedIWord(i) => Word::IWord(i),
                AstConstOperand::SignedFWord(f) => Word::FWord(f),
                AstConstOperand::SWord(s) => s.as_str().into(),
            },
            _ => unreachable!(),
        }
    }
}

macro_rules! binary_operation {
    ($lhs:expr, $op:tt, $rhs:expr) => {
        match ($lhs, $rhs) {
            (Word::IWord(lhs), Word::IWord(rhs)) => Word::IWord(lhs $op rhs),
            (Word::IWord(lhs), Word::FWord(rhs)) => Word::FWord(lhs as f64 $op rhs),
            (Word::FWord(lhs), Word::IWord(rhs)) => Word::FWord(lhs $op rhs as f64),
            (Word::FWord(lhs), Word::FWord(rhs)) => Word::FWord(lhs $op rhs),
            (lhs, rhs) => panic!("Operation not supported between {:?} and {:?}", lhs, rhs),
        }
    };
}

macro_rules! unary_operation {
    ($op:tt, $operand:expr) => {
        match $operand {
            Word::IWord(i) => Word::IWord($op i),
            other => panic!("Operation not supported for {:?}", other),
        }
    };
}

impl std::ops::BitOrAssign for Word {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.same_type_from(self.raw_bits() | rhs.raw_bits());
    }
}

impl std::ops::BitXorAssign for Word {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = self.same_type_from(self.raw_bits() ^ rhs.raw_bits());
    }
}

impl std::ops::BitAndAssign for Word {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.same_type_from(self.raw_bits() & rhs.raw_bits());
    }
}

impl std::ops::AddAssign for Word {
    fn add_assign(&mut self, rhs: Self) {
        *self = binary_operation!(*self, +, rhs);
    }
}

impl std::ops::SubAssign for Word {
    fn sub_assign(&mut self, rhs: Self) {
        *self = binary_operation!(*self, -, rhs);
    }
}

impl std::ops::MulAssign for Word {
    fn mul_assign(&mut self, rhs: Self) {
        *self = binary_operation!(*self, *, rhs);
    }
}

impl std::ops::DivAssign for Word {
    fn div_assign(&mut self, rhs: Self) {
        *self = binary_operation!(*self, /, rhs);
    }
}

impl std::ops::ShlAssign for Word {
    fn shl_assign(&mut self, rhs: Self) {
        *self = self.same_type_from(self.raw_bits() << rhs.raw_bits());
    }
}

impl PartialOrd for Word {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // binary_operation!(self, PartialOrd::partial_cmp, rhs)
        match (self, other) {
            (Word::IWord(lhs), Word::IWord(rhs)) => lhs.partial_cmp(&rhs),
            (Word::IWord(lhs), Word::FWord(rhs)) => (*lhs as f64).partial_cmp(&rhs),
            (Word::FWord(lhs), Word::IWord(rhs)) => lhs.partial_cmp(&(*rhs as f64)),
            (Word::FWord(lhs), Word::FWord(rhs)) => lhs.partial_cmp(&rhs),
            (lhs, rhs) => panic!("Operation not supported between {:?} and {:?}", lhs, rhs),
        }
    }
}

impl std::ops::Neg for Word {
    type Output = Word;

    fn neg(self) -> Self::Output {
        unary_operation!(-, self)
    }
}

impl std::ops::Not for Word {
    type Output = Word;

    fn not(self) -> Self::Output {
        unary_operation!(!, self)
    }
}

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
            .rposition(|foo| *foo == Word::Undefined)
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
