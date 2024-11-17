use super::assembly::Assembly;
use super::ast::{
    Address, AddressOperand, FloatType, IntType, PWord, SWord,
    SimpleAddressOperand as AstSimpleAddressOperand, SourceWord as AstSourceWord,
    StoreOperand as AstStoreOperand,
};
use super::charset::CharSet;

#[derive(Clone, Debug, PartialEq)]
pub enum MemoryWord {
    Undefined,
    Integer(IntType),
    Float(FloatType),
    String(SWord),
    Instruction(PWord),
}

impl MemoryWord {
    // const WORD_MASK: i32 = 0xffffff;

    pub fn to_24_bits(&self) -> i32 {
        match self {
            MemoryWord::Undefined => 0,
            MemoryWord::Integer(i) => *i as i32,
            MemoryWord::Float(_) => unimplemented!(),
            MemoryWord::String(s) => s
                .as_bytes()
                .into_iter()
                .fold(0, |acc, c| acc << 6 | CharSet::char_to_bits(*c).unwrap()),
            MemoryWord::Instruction(_) => unimplemented!(),
        }
    }
}

impl std::ops::BitOrAssign for MemoryWord {
    fn bitor_assign(&mut self, rhs: Self) {
        let result = self.to_24_bits() | rhs.to_24_bits();
        *self = MemoryWord::Integer(result as IntType);
    }
}

impl std::ops::BitXorAssign for MemoryWord {
    fn bitxor_assign(&mut self, rhs: Self) {
        let result = self.to_24_bits() ^ rhs.to_24_bits();
        *self = MemoryWord::Integer(result as IntType);
    }
}

impl std::ops::BitAndAssign for MemoryWord {
    fn bitand_assign(&mut self, rhs: Self) {
        let result = self.to_24_bits() & rhs.to_24_bits();
        *self = MemoryWord::Integer(result as IntType);
    }
}

impl std::ops::AddAssign for MemoryWord {
    fn add_assign(&mut self, rhs: Self) {
        let result = self.to_24_bits() + rhs.to_24_bits();
        *self = MemoryWord::Integer(result as IntType);
    }
}

impl std::ops::SubAssign for MemoryWord {
    fn sub_assign(&mut self, rhs: Self) {
        let result = self.to_24_bits() - rhs.to_24_bits();
        *self = MemoryWord::Integer(result as IntType);
    }
}

impl std::ops::MulAssign for MemoryWord {
    fn mul_assign(&mut self, rhs: Self) {
        let result = self.to_24_bits() * rhs.to_24_bits();
        *self = MemoryWord::Integer(result as IntType);
    }
}

impl std::ops::DivAssign for MemoryWord {
    fn div_assign(&mut self, rhs: Self) {
        let result = self.to_24_bits() / rhs.to_24_bits();
        *self = MemoryWord::Integer(result as IntType);
    }
}

impl std::ops::Neg for MemoryWord {
    type Output = MemoryWord;

    fn neg(self) -> Self::Output {
        match self {
            MemoryWord::Integer(i) => MemoryWord::Integer(-i),
            MemoryWord::Float(f) => MemoryWord::Float(-f),
            _ => panic!("Invalid operand {:?} for Neg", self),
        }
    }
}

impl std::ops::Not for MemoryWord {
    type Output = MemoryWord;

    fn not(self) -> Self::Output {
        match self {
            MemoryWord::Integer(i) => MemoryWord::Integer(!i),
            _ => panic!("Invalid operand {:?} for Neg", self),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Memory(Vec<MemoryWord>);

impl Default for Memory {
    fn default() -> Self {
        Self(vec![MemoryWord::Undefined; 128])
    }
}

impl From<Assembly> for Memory {
    fn from(value: Assembly) -> Self {
        let mut memory = Memory::default();

        let link_address = |address: &Address| match address {
            Address::Identifier(identifier) => Address::NumericAddress(
                value
                    .location(identifier)
                    .expect("Memory::From<Assembly> identifier expected location to be assigned"),
            ),
            Address::NumericAddress(l) => Address::NumericAddress(*l),
        };

        let link_address_operand = |operand: &AddressOperand| {
            let address = match operand.address() {
                AstSimpleAddressOperand::DirectAddress(address) => {
                    AstSimpleAddressOperand::DirectAddress(link_address(&address))
                }
                AstSimpleAddressOperand::IndirectAddress(address) => {
                    AstSimpleAddressOperand::IndirectAddress(link_address(&address))
                }
            };
            AddressOperand::new(address, operand.index())
        };

        let link_store_operand = |operand: &AstStoreOperand| match operand {
            AstStoreOperand::None | AstStoreOperand::ConstOperand(_) => operand.clone(),
            AstStoreOperand::AddressOperand(address_operand) => {
                AstStoreOperand::AddressOperand(link_address_operand(address_operand))
            }
        };

        let link_pword = |pword: &PWord| {
            PWord::new(
                pword.mnemonic(),
                pword.accumulator(),
                link_store_operand(&pword.store_operand()),
            )
        };

        let link = |pword: &AstSourceWord| match pword {
            AstSourceWord::IWord(i) => MemoryWord::Integer(*i),
            AstSourceWord::FWord(f) => MemoryWord::Float(*f),
            AstSourceWord::PWord(pword) => MemoryWord::Instruction(link_pword(pword)),
            AstSourceWord::SWord(s) => MemoryWord::String(s.clone()),
        };

        value
            .code_iter()
            .for_each(|(location, content)| memory[*location] = link(content));

        memory
    }
}

impl std::ops::Index<usize> for Memory {
    type Output = MemoryWord;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
