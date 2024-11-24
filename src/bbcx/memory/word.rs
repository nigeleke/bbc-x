use super::instruction::Instruction;
use super::{FloatType, IntType};

use crate::bbcx::ast::{ConstOperand as AstConstOperand, StoreOperand as AstStoreOperand};
use crate::bbcx::charset::CharSet;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(clippy::enum_variant_names)] // Reflect specification naming
pub enum Word {
    #[default]
    Undefined,
    IWord(IntType),
    FWord(FloatType),
    SWord([u8; 4]),
    PWord(Instruction),
}

// Use u64 to manipulate RawBits, although only the lower 24 bits are relevant.
pub type RawBits = u64;

pub const WORD_SIZE: RawBits = 24;
pub const WORD_MASK: RawBits = (1 << WORD_SIZE) - 1;
pub const OVERFLOW_MASK: RawBits = !WORD_MASK;

const FWORD_SIGN_MASK: RawBits = 0o4000_0000;
const FWORD_EXPONENT_MASK: RawBits = 0o3760_0000;
const FWORD_MANTISSA_MASK: RawBits = 0o0017_7777;

impl Word {
    pub fn iword_from(raw: RawBits) -> Self {
        let i64_value = ((raw as i64) << 40) >> 40;
        Word::IWord(i64_value)
    }

    pub fn fword_from(raw: RawBits) -> Self {
        let sign = if (raw & FWORD_SIGN_MASK) != 0 {
            -1.0
        } else {
            1.0
        };

        let raw_exponent = (raw & FWORD_EXPONENT_MASK) >> 16;
        let exponent = raw_exponent as i32 - 63;

        let raw_mantissa = raw & FWORD_MANTISSA_MASK;
        let mantissa = (raw_mantissa as f64) / ((1 << 17) as f64);

        Word::FWord(sign * (1.0 + mantissa) * (2.0_f64).powi(exponent))
    }

    pub fn sword_from(raw: RawBits) -> Self {
        Word::SWord([
            ((raw & 0o77000000) >> 18) as u8,
            ((raw & 0o00770000) >> 12) as u8,
            ((raw & 0o00007700) >> 6) as u8,
            (raw & 0o00000077) as u8,
        ])
    }

    pub fn pword_from(_raw: RawBits) -> Self {
        unimplemented!()
    }

    pub fn same_type_from(&self, raw: RawBits) -> Self {
        let raw = raw & WORD_MASK;
        match self {
            Word::Undefined => *self,
            Word::IWord(_) => Word::iword_from(raw),
            Word::FWord(_) => Word::fword_from(raw),
            Word::SWord(_) => Word::sword_from(raw),
            Word::PWord(_) => Word::pword_from(raw),
        }
    }

    pub fn raw_bits(&self) -> RawBits {
        (match &self {
            Word::Undefined => 0,
            Word::IWord(i) => *i as RawBits & WORD_MASK,
            Word::FWord(f) => {
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
                let function_code = instruction.function() as RawBits;
                let accumulator = instruction.accumulator() as RawBits;
                let index_register = instruction.index_register() as RawBits;
                let indirect = instruction.indirect() as RawBits;
                let page = instruction.page() as RawBits;
                let address = instruction.address() as RawBits;
                ((function_code << 18)
                    | (accumulator << 15)
                    | (index_register << 12)
                    | (indirect << 11)
                    | (page << 10)
                    | address) as RawBits
            }
        }) as RawBits
    }

    pub fn rotate(&mut self, n: i64) {
        let n = n % 24;
        let raw = (self.raw_bits() as u64) << n;
        let overflow = (raw & OVERFLOW_MASK) >> 24;
        let raw = (raw | overflow) as RawBits & WORD_MASK;
        *self = self.same_type_from(raw)
    }

    pub fn power(&mut self, operand: Word) {
        *self = match (&self, operand) {
            (Word::IWord(lhs), Word::IWord(rhs)) => Word::IWord(lhs.pow(rhs as u32)),
            (Word::IWord(lhs), Word::FWord(rhs)) => Word::FWord((*lhs as f64).powf(rhs)),
            (Word::FWord(lhs), Word::IWord(rhs)) => Word::FWord(lhs.powf(rhs as f64)),
            (Word::FWord(lhs), Word::FWord(rhs)) => Word::FWord(lhs.powf(rhs)),
            (lhs, rhs) => panic!("Operation not supported between {:?} and {:?}", lhs, rhs),
        };
    }
}

impl From<i64> for Word {
    fn from(value: i64) -> Self {
        Word::iword_from(value as RawBits)
    }
}

impl From<f64> for Word {
    fn from(value: f64) -> Self {
        Word::FWord(value)
    }
}

impl From<&str> for Word {
    fn from(value: &str) -> Self {
        assert!(
            value.len() <= 4,
            "String too long. Expected length <= 4 bytes"
        );

        let mut buffer = [0u8; 4];
        let start_index = 4 - value.len();

        for (i, &c) in value.as_bytes().iter().enumerate() {
            buffer[start_index + i] =
                CharSet::char_to_bits(c).expect("Invalid character for conversion") as u8;
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
            (Word::IWord(lhs), Word::IWord(rhs)) => lhs.partial_cmp(rhs),
            (Word::IWord(lhs), Word::FWord(rhs)) => (*lhs as f64).partial_cmp(rhs),
            (Word::FWord(lhs), Word::IWord(rhs)) => lhs.partial_cmp(&(*rhs as f64)),
            (Word::FWord(lhs), Word::FWord(rhs)) => lhs.partial_cmp(rhs),
            (lhs, rhs) => panic!("Operation not supported between {:?} and {:?}", lhs, rhs),
        }
    }
}

impl std::ops::Neg for Word {
    type Output = Word;

    fn neg(self) -> Self::Output {
        match self {
            Word::IWord(i) => Word::IWord(-i),
            Word::FWord(f) => Word::FWord(-f),
            _ => panic!("NEG operation not supported for {:?}", self),
        }
    }
}

impl std::ops::Not for Word {
    type Output = Word;

    fn not(self) -> Self::Output {
        let raw = self.raw_bits();
        let not_raw = !raw;
        println!("raw: {:#010o} not: {:#010o}", raw, not_raw);
        self.same_type_from(not_raw)
    }
}
