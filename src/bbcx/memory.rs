use super::ast::{
    Address as AstAddress, Identifier, Location as AstLocation,
    SimpleAddressOperand as AstSimpleAddressOperand, SourceProgramWord as AstSourceProgramWord,
    StoreOperand as AstStoreOperand,
};
use super::{
    assembly::{Assembly, Symbols},
    ast::*,
};

#[derive(Clone, Debug, PartialEq)]
pub enum MemoryWord {
    Undefined,
    Integer(IntType),
    Float(FloatType),
    String(SWord),
    Instruction(PWord),
}

impl MemoryWord {
    const WORD_MASK: u32 = 0xffffff;

    fn to_24_bits(&self) -> u32 {
        match self {
            MemoryWord::Undefined => 0,
            MemoryWord::Integer(i) => (*i as u32) & MemoryWord::WORD_MASK,
            MemoryWord::Float(_) => unimplemented!(),
            MemoryWord::String(_) => unimplemented!(),
            MemoryWord::Instruction(_) => unimplemented!(),
        }
    }
}

impl std::ops::BitOrAssign for MemoryWord {
    fn bitor_assign(&mut self, rhs: Self) {
        let ored = self.to_24_bits() | rhs.to_24_bits();
        *self = MemoryWord::Integer(ored as IntType);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Memory(Vec<MemoryWord>);

impl Memory {
    pub fn content(&self, offset: usize) -> MemoryWord {
        self.0[offset].clone()
    }
}

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
                    .location(&identifier)
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

        let link = |pword: &AstSourceProgramWord| match pword {
            AstSourceProgramWord::IWord(i) => MemoryWord::Integer(*i),
            AstSourceProgramWord::FWord(f) => MemoryWord::Float(*f),
            AstSourceProgramWord::PWord(pword) => MemoryWord::Instruction(link_pword(pword)),
            AstSourceProgramWord::SWord(s) => MemoryWord::String(s.clone()),
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
