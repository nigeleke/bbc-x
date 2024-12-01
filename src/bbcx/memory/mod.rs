mod convert;
mod instruction;
mod result;
mod state;
mod word;

pub use self::convert::word_to_instruction;
pub use self::instruction::{
    Accumulator, Address, Function, IndexRegister, Instruction, MemoryIndex,
};
pub use self::state::State as Memory;
pub use self::word::{ops::*, *};

#[cfg(test)]
pub use self::convert::instruction_to_word;

#[cfg(test)]
pub use self::instruction::Builder as InstructionBuilder;

#[cfg(test)]
pub use self::state::MEMORY_SIZE;
