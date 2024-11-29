use std::ops::AddAssign;

use super::result::{Error, Result};

pub type Function = crate::bbcx::ast::Mnemonic;

pub trait AsBits {
    fn as_bits(&self) -> u32;
}

pub trait MemoryRef {
    type Target: TryFrom<usize>;

    fn index(&self) -> usize;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Accumulator(usize);

impl AsBits for Accumulator {
    fn as_bits(&self) -> u32 {
        self.0 as u32
    }
}

impl MemoryRef for Accumulator {
    type Target = Accumulator;
    fn index(&self) -> usize {
        self.0
    }
}

impl TryFrom<usize> for Accumulator {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self> {
        (value <= Instruction::ACCUMULATOR_LIMIT)
            .then_some(Accumulator(value))
            .ok_or(Error::InvalidAccumulator(value))
    }
}

impl std::ops::Add<usize> for Accumulator {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        (self.0 + rhs)
            .try_into()
            .unwrap_or_else(|err| panic!("{} using: {} + {}", err, self.0, rhs))
    }
}

impl std::ops::Sub<usize> for Accumulator {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        (self.0 - rhs)
            .try_into()
            .unwrap_or_else(|err| panic!("{} using: {} - {}", err, self.0, rhs))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IndexRegister(Accumulator);

impl IndexRegister {
    pub fn is_indexable(&self) -> bool {
        self.0 .0 != 0
    }
}

impl AsBits for IndexRegister {
    fn as_bits(&self) -> u32 {
        self.0.as_bits()
    }
}

impl MemoryRef for IndexRegister {
    type Target = IndexRegister;

    fn index(&self) -> usize {
        self.0.index()
    }
}

impl TryFrom<usize> for IndexRegister {
    type Error = Error;

    fn try_from(value: usize) -> std::result::Result<Self, Self::Error> {
        value.try_into().map(IndexRegister)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Indirect(bool);

impl AsBits for Indirect {
    fn as_bits(&self) -> u32 {
        self.0 as u32
    }
}

impl From<bool> for Indirect {
    fn from(value: bool) -> Self {
        Indirect(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Page(usize);

impl AsBits for Page {
    fn as_bits(&self) -> u32 {
        self.0 as u32
    }
}

impl TryFrom<usize> for Page {
    type Error = Error;

    #[allow(clippy::absurd_extreme_comparisons)]
    fn try_from(value: usize) -> std::result::Result<Self, Self::Error> {
        (value <= Instruction::PAGE_LIMIT)
            .then_some(Page(value))
            .ok_or(Error::InvalidPage(value))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Address(usize);

impl AsBits for Address {
    fn as_bits(&self) -> u32 {
        self.0 as u32
    }
}

impl MemoryRef for Address {
    type Target = Address;

    fn index(&self) -> usize {
        self.0
    }
}

impl TryFrom<usize> for Address {
    type Error = Error;

    fn try_from(value: usize) -> std::result::Result<Self, Self::Error> {
        (value <= Instruction::ADDRESS_LIMIT)
            .then_some(Address(value))
            .ok_or(Error::InvalidAddress(value))
    }
}

impl AddAssign<isize> for Address {
    fn add_assign(&mut self, rhs: isize) {
        self.0 = (self.index() as isize + rhs) as usize
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Instruction {
    function: Function,
    accumulator: Accumulator,
    index_register: IndexRegister,
    indirect: Indirect,
    page: Page,
    address: Address,
}

impl Instruction {
    const ACCUMULATOR_LIMIT: usize = (1 << 3) - 1;
    const ADDRESS_LIMIT: usize = (1 << 10) - 1;
    const PAGE_LIMIT: usize = 0;

    pub fn new(function: Function) -> Self {
        Self {
            function,
            ..Default::default()
        }
    }

    pub fn function(&self) -> Function {
        self.function
    }

    pub fn accumulator(&self) -> Accumulator {
        self.accumulator
    }

    pub fn index_register(&self) -> IndexRegister {
        self.index_register
    }

    pub fn indirect(&self) -> Indirect {
        self.indirect
    }

    pub fn is_indirect(&self) -> bool {
        self.indirect.0
    }

    pub fn page(&self) -> Page {
        self.page
    }

    pub fn address(&self) -> Address {
        self.address
    }
}

#[derive(Default)]
pub struct Builder {
    instruction: Instruction,
}

impl Builder {
    pub fn new(function: Function) -> Self {
        let instruction = Instruction::new(function);
        Self { instruction }
    }

    pub fn with_accumulator<T>(mut self, accumulator: T) -> Self
    where
        T: TryInto<Accumulator>,
        T::Error: std::fmt::Debug,
    {
        self.instruction.accumulator = accumulator.try_into().expect("valid accumulator required");
        self
    }

    pub fn with_index_register<T>(mut self, index_register: T) -> Self
    where
        T: TryInto<IndexRegister>,
        T::Error: std::fmt::Debug,
    {
        self.instruction.index_register = index_register
            .try_into()
            .expect("valid index register required");
        self
    }

    pub fn with_indirect<T>(mut self, indirect: T) -> Self
    where
        T: TryInto<Indirect>,
        T::Error: std::fmt::Debug,
    {
        self.instruction.indirect = indirect.try_into().expect("valid indirect register");
        self
    }

    pub fn with_page<T>(mut self, page: T) -> Self
    where
        T: TryInto<Page>,
        T::Error: std::fmt::Debug,
    {
        self.instruction.page = page.try_into().expect("valid page required");
        self
    }

    pub fn with_address<T>(mut self, address: T) -> Self
    where
        T: TryInto<Address>,
        T::Error: std::fmt::Debug,
    {
        self.instruction.address = address.try_into().expect("valid address required");
        self
    }

    pub fn build(&self) -> Instruction {
        self.instruction
    }
}
