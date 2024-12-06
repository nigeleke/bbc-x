mod convert;
mod instruction;
mod result;
mod state;
mod word;

pub use self::convert::{decrement, increment, instruction_to_word, word_to_instruction};
pub use self::instruction::{
    Accumulator, Address, Builder as InstructionBuilder, Function, IndexRegister, Instruction,
    MemoryIndex,
};
pub use self::state::State as Memory;
pub use self::word::{ops::*, *};

#[cfg(test)]
pub use self::state::MEMORY_SIZE;
