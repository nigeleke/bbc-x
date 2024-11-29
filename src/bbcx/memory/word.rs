use super::result::{Error, Result};

use crate::bbcx::charset::CharSet;

use num_enum::TryFromPrimitive;

// The word's rawbits are u24, which could be held in a u32.
// We use u64 to aide double length operations..
type RawBits = u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub(crate) enum WordType {
    #[default]
    Undefined,
    IWord,
    FWord,
    SWord,
    PWord,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Word {
    word_type: WordType,
    raw_bits: RawBits,
}

impl Word {
    const SIZE: usize = 24;
    const MASK: RawBits = (1 << Word::SIZE) - 1;
    const OVERFLOW_MASK: RawBits = !Word::MASK;
    const SIZE2: usize = Word::SIZE + Word::SIZE;
    const MASK2: RawBits = (1 << Word::SIZE2) - 1;
    const OVERFLOW_MASK2: RawBits = !Word::MASK2;

    const FWORD_SIGN_MASK: RawBits = 0o4000_0000;
    const FWORD_EXPONENT_MASK: RawBits = 0o3760_0000;
    const FWORD_EXPONENT_BIAS: i32 = 63;
    const FWORD_MANTISSA_MASK: RawBits = 0o0017_7777;

    const F64_SIGN_MASK: RawBits = 0x8000_0000_0000_0000;
    const F64_EXPONENT_MASK: RawBits = 0x7FF0_0000_0000_0000;
    const F64_EXPONENT_BIAS: i32 = 1023;
    const F64_MANTISSA_MASK: RawBits = 0x000F_FFFF_FFFF_FFFF;

    const SWORD_CHAR_MASK: RawBits = 0o77;
    const SWORD_CHAR_SIZE: usize = Self::SWORD_CHAR_MASK.trailing_ones() as usize;
    const SWORD_MAX_CHARS: usize = Word::SIZE / Word::SWORD_CHAR_SIZE;

    pub(crate) const PWORD_FUNCTION_MASK: RawBits = 0o7700_0000;
    pub(crate) const PWORD_ACCUMULATOR_MASK: RawBits = 0o0070_0000;
    pub(crate) const PWORD_INDEX_REGISTER_MASK: RawBits = 0o0007_0000;
    pub(crate) const PWORD_INDIRECT_MASK: RawBits = 0o0000_4000;
    pub(crate) const PWORD_PAGE_MASK: RawBits = 0o0000_2000;
    pub(crate) const PWORD_ADDRESS_MASK: RawBits = 0o0000_1777;

    pub(crate) fn new(word_type: WordType, raw_bits: RawBits) -> Self {
        Self {
            word_type,
            raw_bits,
        }
    }

    pub fn is_undefined(&self) -> bool {
        self.word_type == WordType::Undefined
    }

    pub fn is_instruction(&self) -> bool {
        self.word_type == WordType::PWord
    }

    pub fn pword_function_bits(&self) -> RawBits {
        assert!(self.word_type == WordType::PWord);
        bits::get(self.raw_bits, Word::PWORD_FUNCTION_MASK) as RawBits
    }

    pub fn pword_accumulator_bits(&self) -> RawBits {
        assert!(self.word_type == WordType::PWord);
        bits::get(self.raw_bits, Word::PWORD_ACCUMULATOR_MASK) as RawBits
    }

    pub fn pword_index_register_bits(&self) -> RawBits {
        assert!(self.word_type == WordType::PWord);
        bits::get(self.raw_bits, Word::PWORD_INDEX_REGISTER_MASK) as RawBits
    }

    pub fn pword_indirect_bits(&self) -> RawBits {
        assert!(self.word_type == WordType::PWord);
        bits::get(self.raw_bits, Word::PWORD_INDIRECT_MASK) as RawBits
    }

    pub fn pword_page_bits(&self) -> RawBits {
        assert!(self.word_type == WordType::PWord);
        bits::get(self.raw_bits, Word::PWORD_PAGE_MASK) as RawBits
    }

    pub fn pword_address_bits(&self) -> RawBits {
        assert!(self.word_type == WordType::PWord);
        bits::get(self.raw_bits, Word::PWORD_ADDRESS_MASK) as RawBits
    }

    pub fn as_i64(&self) -> Result<i64> {
        (self.word_type == WordType::IWord)
            .then_some((self.raw_bits as i64) << 40 >> 40)
            .ok_or(Error::CannotConvertFromWord(format!("i64 from {}", self)))
    }

    pub fn as_f64(&self) -> Result<f64> {
        (self.word_type == WordType::FWord)
            .then_some({
                let raw_bits = self.raw_bits;

                let zero_bits = raw_bits & (Word::FWORD_EXPONENT_MASK | Word::FWORD_MANTISSA_MASK);

                if zero_bits != 0 {
                    let sign = bits::get(raw_bits, Word::FWORD_SIGN_MASK);
                    let exponent = bits::get(raw_bits, Word::FWORD_EXPONENT_MASK) as i32
                        - Word::FWORD_EXPONENT_BIAS;
                    let mantissa = bits::get(raw_bits, Word::FWORD_MANTISSA_MASK);

                    let f64_exponent = exponent + Word::F64_EXPONENT_BIAS;
                    let f64_mantissa = mantissa
                        << (Word::F64_MANTISSA_MASK.trailing_ones()
                            - Word::FWORD_MANTISSA_MASK.trailing_ones());

                    let bits = bits::set(sign, Word::F64_SIGN_MASK)
                        | bits::set(f64_exponent as u64, Word::F64_EXPONENT_MASK)
                        | bits::set(f64_mantissa, Word::F64_MANTISSA_MASK);

                    f64::from_bits(bits)
                } else {
                    0.0
                }
            })
            .ok_or(Error::CannotConvertFromWord(format!("f64 from {}", self)))
    }

    pub fn as_char(&self) -> Result<u8> {
        match self.word_type {
            WordType::IWord | WordType::SWord => {
                let bits = bits::get(self.raw_bits, Word::SWORD_CHAR_MASK);
                let char = CharSet::bits_to_char(bits)
                    .unwrap_or_else(|| panic!("Invalid character {}", bits));
                Ok(char)
            }
            _ => Err(Error::CannotConvertFromWord(format!("u8 from {}", self))),
        }
    }

    pub fn word_type(&self) -> Word {
        Word::new(
            WordType::IWord,
            match self.word_type {
                WordType::IWord => 0,
                WordType::FWord => 1,
                WordType::SWord => 2,
                WordType::PWord => 3,
                _ => panic!("Word type: Undefined"),
            },
        )
    }

    pub fn set_word_type(&mut self, operand: &Word) {
        let u32_word_type = ((operand.raw_bits & 0b11) + 1) as u32;
        self.word_type = WordType::try_from_primitive(u32_word_type).unwrap();
    }

    pub fn word_bits(&self) -> Word {
        Word::new(WordType::IWord, self.raw_bits)
    }

    pub fn set_word_bits(&mut self, operand: &Word) {
        self.raw_bits = operand.raw_bits;
    }

    pub fn rotate(&mut self, n: i64) {
        let mut n = n % (Word::SIZE as i64);
        if n < 0 {
            n += Word::SIZE as i64
        };

        let shifted_bits = self.raw_bits << n;
        let overflow = bits::get(shifted_bits, Word::OVERFLOW_MASK);

        self.raw_bits = shifted_bits & Word::MASK | overflow as RawBits;
    }

    pub fn power(&mut self, other: &Word) {
        match (self.word_type, other.word_type) {
            (WordType::IWord, WordType::IWord) => {
                let x = self.as_i64().unwrap();
                let n = other.as_i64().unwrap();
                let result = x.pow(n as u32);
                *self = result.try_into().unwrap();
            }
            (WordType::IWord, WordType::FWord) => {
                let x = self.as_i64().unwrap();
                let n = other.as_f64().unwrap();
                let result = (x as f64).powf(n);
                *self = if n == 0.0 {
                    (result as i64).try_into().unwrap()
                } else {
                    result.try_into().unwrap()
                };
            }
            (WordType::FWord, WordType::IWord) => {
                let x = self.as_f64().unwrap();
                let n = other.as_i64().unwrap();
                let result = x.powi(n as i32);
                *self = if n == 0 {
                    (result as i64).try_into().unwrap()
                } else {
                    result.try_into().unwrap()
                };
            }
            (WordType::FWord, WordType::FWord) => {
                let x = self.as_f64().unwrap();
                let n = other.as_f64().unwrap();
                let result = x.powf(n);
                *self = if n == 0.0 {
                    (result as i64).try_into().unwrap()
                } else {
                    result.try_into().unwrap()
                };
            }
            (lhs, rhs) => panic!("Operation not supported between {:?} and {:?}", lhs, rhs),
        };
    }
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:#010o}", self.word_type, self.raw_bits)
    }
}

impl TryFrom<i64> for Word {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self> {
        let raw_value = value & Word::MASK as i64;
        let word = Word::new(WordType::IWord, raw_value as RawBits);
        (word.as_i64().unwrap() == value)
            .then_some(word)
            .ok_or(Error::InvalidIWordValue(value))
    }
}

impl TryFrom<f64> for Word {
    type Error = Error;

    fn try_from(value: f64) -> Result<Self> {
        fn can_represent_f24(value: f64) -> bool {
            let bad_initial_value = value.is_nan() || value.is_infinite();

            let (_sign, exponent, _mantissa) = decompose_f64(value);

            let valid_exponent =
                (-Word::FWORD_EXPONENT_BIAS - 1..=Word::FWORD_EXPONENT_BIAS).contains(&exponent);

            !bad_initial_value && valid_exponent
        }

        fn decompose_f64(value: f64) -> (u8, i32, u64) {
            let bits = value.to_bits();

            let sign = bits::get(bits, Word::F64_SIGN_MASK) as u8;
            let exponent =
                bits::get(bits, Word::F64_EXPONENT_MASK) as i32 - Word::F64_EXPONENT_BIAS;
            let mantissa = bits::get(bits, Word::F64_MANTISSA_MASK);
            (sign, exponent, mantissa)
        }

        fn convert_to_f24(value: f64) -> Option<RawBits> {
            can_represent_f24(value).then_some({
                let (sign, exponent, mantissa) = decompose_f64(value);

                let f24_exponent = exponent + Word::FWORD_EXPONENT_BIAS;
                let f24_mantissa = mantissa
                    >> (Word::F64_MANTISSA_MASK.trailing_ones()
                        - Word::FWORD_MANTISSA_MASK.trailing_ones());

                bits::set(sign, Word::FWORD_SIGN_MASK)
                    | bits::set(f24_exponent as u64, Word::FWORD_EXPONENT_MASK)
                    | bits::set(f24_mantissa, Word::FWORD_MANTISSA_MASK) as RawBits
            })
        }

        if value != 0.0 {
            convert_to_f24(value)
                .map(|raw_bits| Word::new(WordType::FWord, raw_bits))
                .ok_or(Error::InvalidFWordValue(value))
        } else {
            Ok(Word::new(WordType::FWord, 0))
        }
    }
}

impl TryFrom<&str> for Word {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        (value.len() <= Word::SWORD_MAX_CHARS)
            .then_some(Word::new(
                WordType::SWord,
                value
                    .as_bytes()
                    .iter()
                    .map(|c| CharSet::char_to_bits(*c).unwrap())
                    .fold(RawBits::default(), |raw, c| {
                        (raw << Word::SWORD_CHAR_SIZE) | c as RawBits
                    }),
            ))
            .ok_or(Error::InvalidSWordValue(value.to_string()))
    }
}

pub mod ops {
    use super::*;

    impl std::ops::BitOrAssign for Word {
        fn bitor_assign(&mut self, rhs: Self) {
            self.raw_bits |= rhs.raw_bits;
        }
    }

    impl std::ops::BitXorAssign for Word {
        fn bitxor_assign(&mut self, rhs: Self) {
            self.raw_bits ^= rhs.raw_bits;
        }
    }

    impl std::ops::BitAndAssign for Word {
        fn bitand_assign(&mut self, rhs: Self) {
            self.raw_bits &= rhs.raw_bits;
        }
    }

    macro_rules! binary_operation {
        ($lhs:expr, $op:tt, $rhs:expr) => {
            match ($lhs.word_type, $rhs.word_type) {
                (WordType::IWord, WordType::IWord) => ($lhs.as_i64().unwrap() $op $rhs.as_i64().unwrap()).try_into(),
                (WordType::IWord, WordType::FWord) => ($lhs.as_i64().unwrap() as f64 $op $rhs.as_f64().unwrap()).try_into(),
                (WordType::FWord, WordType::IWord) => ($lhs.as_f64().unwrap() $op $rhs.as_i64().unwrap() as f64).try_into(),
                (WordType::FWord, WordType::FWord) => ($lhs.as_f64().unwrap() $op $rhs.as_f64().unwrap()).try_into(),
                (lhs, rhs) => panic!("Operation not supported between {:?} and {:?}", lhs, rhs),
            }.expect("Operation $op failed")
        }
    }

    impl std::ops::AddAssign for Word {
        fn add_assign(&mut self, rhs: Self) {
            *self = binary_operation!(self, +, rhs);
        }
    }

    impl std::ops::SubAssign for Word {
        fn sub_assign(&mut self, rhs: Self) {
            *self = binary_operation!(self, -, rhs);
        }
    }

    impl std::ops::MulAssign for Word {
        fn mul_assign(&mut self, rhs: Self) {
            *self = binary_operation!(self, *, rhs);
        }
    }

    impl std::ops::DivAssign for Word {
        fn div_assign(&mut self, rhs: Self) {
            *self = binary_operation!(self, /, rhs);
        }
    }

    impl std::ops::Neg for Word {
        type Output = Word;

        fn neg(self) -> Self::Output {
            match self.word_type {
                WordType::IWord => self
                    .as_i64()
                    .and_then(|i| (-i).try_into())
                    .unwrap_or_else(|err| panic!("NEG failed: {} using: {}", err, self)),
                WordType::FWord => self
                    .as_f64()
                    .and_then(|f| (-f).try_into())
                    .unwrap_or_else(|err| panic!("NEG failed: {} using: {}", err, self)),
                _ => panic!("NEG operation not supported for {:?}", self),
            }
        }
    }

    impl std::ops::Not for Word {
        type Output = Word;

        fn not(self) -> Self::Output {
            Word::new(self.word_type, (!self.raw_bits) & Word::MASK)
        }
    }

    impl PartialOrd for Word {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            match (self.word_type, other.word_type) {
                (WordType::IWord, WordType::IWord) => {
                    self.as_i64().unwrap().partial_cmp(&other.as_i64().unwrap())
                }
                (WordType::IWord, WordType::FWord) => {
                    (self.as_i64().unwrap() as f64).partial_cmp(&other.as_f64().unwrap())
                }
                (WordType::FWord, WordType::IWord) => self
                    .as_f64()
                    .unwrap()
                    .partial_cmp(&(other.as_i64().unwrap() as f64)),
                (WordType::FWord, WordType::FWord) => {
                    self.as_f64().unwrap().partial_cmp(&other.as_f64().unwrap())
                }
                (lhs, rhs) => panic!("Operation not supported between {:?} and {:?}", lhs, rhs),
            }
        }
    }

    impl std::ops::ShlAssign for Word {
        fn shl_assign(&mut self, rhs: Self) {
            let n = rhs.as_i64().unwrap();
            self.raw_bits <<= n;
            self.raw_bits &= Word::MASK;
        }
    }

    fn msw_lsw_to_u64(msw: &Word, lsw: &Word) -> u64 {
        msw.raw_bits << Word::SIZE | (lsw.raw_bits & Word::MASK)
    }

    fn u64_to_msw_lsw(value: u64) -> (Word, Word) {
        let msw = bits::get(value, Word::OVERFLOW_MASK as i64 as u64);
        let lsw = bits::get(value, Word::MASK);

        (
            Word::new(WordType::Undefined, msw as RawBits & Word::MASK),
            Word::new(WordType::Undefined, lsw as RawBits & Word::MASK),
        )
    }

    fn msw_lsw_to_i64(msw: &Word, lsw: &Word) -> i64 {
        let signed_msw = msw.as_i64().unwrap() as u64;
        ((signed_msw << Word::SIZE) | lsw.raw_bits) as i64
    }

    fn i64_to_msw_lsw(value: i64) -> (Word, Word) {
        u64_to_msw_lsw(value as u64)
    }

    pub fn double_shift_left(msw: &Word, lsw: &Word, operand: &Word) -> Result<(Word, Word)> {
        let n: i64 = operand.as_i64()?;
        let lhs: u64 = msw_lsw_to_u64(msw, lsw);
        let shifted_lhs = lhs << n;
        Ok(u64_to_msw_lsw(shifted_lhs))
    }

    pub fn double_rotate_left(msw: &Word, lsw: &Word, operand: &Word) -> Result<(Word, Word)> {
        let mut n: i64 = operand.as_i64()? % Word::SIZE2 as i64;
        if n < 0 {
            n += Word::SIZE2 as i64
        };

        let lhs = msw_lsw_to_u64(msw, lsw);

        let shifted_bits = lhs << n;
        let overflow = bits::get(shifted_bits, Word::OVERFLOW_MASK2);
        let raw = shifted_bits & Word::MASK2 | overflow;

        Ok(u64_to_msw_lsw(raw))
    }

    pub fn double_mult(msw: &Word, lsw: &Word, operand: &Word) -> Result<(Word, Word)> {
        let lhs: i64 = msw_lsw_to_i64(msw, lsw);
        let result = lhs * operand.as_i64().unwrap();
        Ok(i64_to_msw_lsw(result))
    }

    pub fn double_div(msw: &Word, lsw: &Word, operand: &Word) -> Result<(Word, Word)> {
        let lhs: i64 = msw_lsw_to_i64(msw, lsw);
        let result = lhs / operand.as_i64().unwrap();
        Ok(i64_to_msw_lsw(result))
    }

    pub fn squash(msw: &Word, lsw: &Word) -> Result<(Word, Word)> {
        let sign = msw.as_i64().unwrap();
        let lhs = lsw.as_i64().unwrap();

        assert!(if lhs >= 0 { sign == 0 } else { sign == -1 });
        Ok(i64_to_msw_lsw(lhs))
    }
}

pub mod bits {
    pub fn get<V, M>(value: V, mask: M) -> u64
    where
        V: 'static + Into<u64>,
        M: 'static + Into<u64>,
    {
        let value: u64 = value.into();
        let mask: u64 = mask.into();
        let shift = mask.trailing_zeros();
        (value & mask) >> shift
    }

    pub fn set<V, M>(value: V, mask: M) -> u64
    where
        V: 'static + Into<u64>,
        M: 'static + Into<u64>,
    {
        let value: u64 = value.into();
        let mask: u64 = mask.into();
        let shift = mask.trailing_zeros();
        (value & (mask >> shift)) << shift
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn will_create_undefined_word_by_default() {
        assert_eq!(Word::default(), Word::new(WordType::Undefined, 0))
    }

    #[test]
    fn will_convert_from_int() {
        let test_int = |i, expected| {
            let result = Word::try_from(i);
            assert_eq!(result, Ok(Word::new(WordType::IWord, expected)));
            assert_eq!(result.unwrap().as_i64(), Ok(i));
        };
        test_int(42, 0o00000052);
        test_int(-42, 0o77777726);
    }

    #[test]
    fn will_not_convert_from_int_when_out_of_range() {
        let value = 10_000_000;
        assert_eq!(Word::try_from(value), Err(Error::InvalidIWordValue(value)))
    }

    #[test]
    fn will_convert_from_float() {
        let test_float = |f, expected| {
            let result = Word::try_from(f);
            assert_eq!(result, Ok(Word::new(WordType::FWord, expected)));
            assert_eq!(result.unwrap().as_f64(), Ok(f));
        };
        test_float(42.0, 0o21050000);
        test_float(-42.0, 0o61050000);
        test_float(0.0, 0o00000000);
        test_float(1.0, 0o17600000);
    }

    #[test]
    fn will_not_convert_from_float_when_out_of_range() {
        let value = 1844672.0e19;
        assert_eq!(Word::try_from(value), Err(Error::InvalidFWordValue(value)));
    }

    #[test]
    fn will_convert_from_str() {
        let test_string = |s: String, expected| {
            let result = Word::try_from(s.as_str());
            assert_eq!(result, Ok(Word::new(WordType::SWord, expected)));
            let raw_bits = result.unwrap().raw_bits;
            assert_eq!(raw_bits, raw_bits & Word::MASK);
        };
        test_string("ABCD".to_string(), 0o01020304);
        test_string("AB".to_string(), 0o00000102);
        test_string("AB\0\0".to_string(), 0o01020000);
        test_string("WXYZ".to_string(), 0o27303132);
    }

    #[test]
    fn will_not_convert_from_str_when_out_of_range() {
        let value = "ABCDEF";
        assert_eq!(
            Word::try_from(value),
            Err(Error::InvalidSWordValue(value.into()))
        );
    }
}
