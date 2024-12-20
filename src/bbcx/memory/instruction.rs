use super::result::{Error, Result};

pub type Function = crate::bbcx::ast::Mnemonic;

pub trait MemoryIndex {
    fn memory_index(&self) -> usize;
}

pub trait Bits {
    fn bits(&self) -> u32;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Accumulator(Address);

impl Accumulator {
    const DEFAULT: usize = 1;
}

impl Default for Accumulator {
    fn default() -> Self {
        Self::DEFAULT.try_into().unwrap()
    }
}

impl MemoryIndex for Accumulator {
    fn memory_index(&self) -> usize {
        self.0.memory_index()
    }
}

impl Bits for Accumulator {
    fn bits(&self) -> u32 {
        self.0.bits()
    }
}

impl TryFrom<usize> for Accumulator {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self> {
        (value <= Instruction::ACCUMULATOR_LIMIT)
            .then_some(Address::try_from(value)?)
            .map(Accumulator)
            .ok_or(Error::InvalidAccumulator(value))
    }
}

impl std::ops::Add<isize> for Accumulator {
    type Output = Self;

    fn add(self, rhs: isize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl std::ops::Sub<isize> for Accumulator {
    type Output = Self;

    fn sub(self, rhs: isize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl std::fmt::Display for Accumulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<2}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IndexRegister(Accumulator);

impl IndexRegister {
    pub fn is_indexable(&self) -> bool {
        let zero = 0.try_into().unwrap();
        self.0 != zero
    }
}

impl Default for IndexRegister {
    fn default() -> Self {
        Self(0.try_into().unwrap())
    }
}

impl MemoryIndex for IndexRegister {
    fn memory_index(&self) -> usize {
        self.0.memory_index()
    }
}

impl Bits for IndexRegister {
    fn bits(&self) -> u32 {
        self.0.bits()
    }
}

impl TryFrom<usize> for IndexRegister {
    type Error = Error;

    fn try_from(value: usize) -> std::result::Result<Self, Self::Error> {
        value.try_into().map(IndexRegister)
    }
}

impl std::fmt::Display for IndexRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let index = self.0.memory_index();

        let formatted_index = if index != 0 {
            &format!("({})", index)
        } else {
            ""
        };

        write!(f, "{}", formatted_index)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Indirect(bool);

impl Bits for Indirect {
    fn bits(&self) -> u32 {
        self.0 as u32
    }
}

impl From<bool> for Indirect {
    fn from(value: bool) -> Self {
        Indirect(value)
    }
}

impl std::fmt::Display for Indirect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if self.0 { "*" } else { "" })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Page(usize);

impl Bits for Page {
    fn bits(&self) -> u32 {
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

impl MemoryIndex for Address {
    fn memory_index(&self) -> usize {
        self.0
    }
}

impl Bits for Address {
    fn bits(&self) -> u32 {
        self.0 as u32
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

impl std::ops::Add<isize> for Address {
    type Output = Address;

    fn add(self, rhs: isize) -> Self::Output {
        Self((self.0 as isize + rhs) as usize)
    }
}

impl std::ops::Sub<isize> for Address {
    type Output = Address;

    fn sub(self, rhs: isize) -> Self::Output {
        Self((self.0 as isize - rhs) as usize)
    }
}

impl std::ops::AddAssign<isize> for Address {
    fn add_assign(&mut self, rhs: isize) {
        self.0 = (self.0 as isize + rhs) as usize
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<04}", self.0)
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

    pub fn set_address(&mut self, address: Address) {
        self.address = address;
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}, {}{}{}",
            self.function(),
            self.accumulator(),
            self.indirect(),
            self.address(),
            self.index_register()
        )
    }
}

#[derive(Default, Debug)]
pub struct Builder {
    instruction: Instruction,
}

impl Builder {
    pub fn new(function: Function) -> Self {
        let mut instruction = Instruction::new(function);
        instruction.accumulator = Accumulator::default();
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
