use super::{Function, IndexRegister, Offset};

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

    pub fn page(&self) -> usize {
        self.page
    }

    pub fn address(&self) -> Offset {
        self.address
    }
}
