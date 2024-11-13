use std::collections::HashMap;

use super::assembly::Assembly;
use super::ast::*;
use super::memory::*;

use crate::result::Result;

#[derive(Debug, PartialEq)]
pub struct Executor {
    execution_context: ExecutionContext,
}

impl Executor {
    pub fn execute(assembly: Assembly) -> Result<ExecutionContext> {
        let mut executor = Executor::new(assembly);
        while executor.can_step() {
            executor.step();
        }
        Ok(executor.execution_context)
    }

    fn new(assembly: Assembly) -> Self {
        Self {
            execution_context: assembly.into(),
        }
    }

    fn can_step(&self) -> bool {
        let context = &self.execution_context;
        let content = &context.memory[context.program_counter];
        matches!(content, MemoryWord::Instruction(_))
    }

    fn step(&mut self) {
        let program_counter = self.execution_context.program_counter;
        self.execution_context.program_counter += 1;
        match self.execution_context.memory[program_counter].clone() {
            MemoryWord::Instruction(pword) => self.step_word(&pword),
            _ => unreachable!(),
        }
    }

    fn step_word(&mut self, pword: &PWord) {
        let acc = pword.accumulator().as_usize();
        let store_operand = pword.store_operand();
        let store_operand = self.value_at(&store_operand);

        type ExecFn = fn(&mut Executor, acc: Location, operand: MemoryWord);
        let execution: HashMap<Mnemonic, ExecFn> = vec![
            (Mnemonic::NIL, Executor::exec_nil as ExecFn),
            (Mnemonic::OR, Executor::exec_or as ExecFn),
            (Mnemonic::NEQV, Executor::exec_neqv as ExecFn),
            (Mnemonic::AND, Executor::exec_and as ExecFn),
            (Mnemonic::ADD, Executor::exec_add as ExecFn),
            (Mnemonic::SUBT, Executor::exec_subt as ExecFn),
            (Mnemonic::MULT, Executor::exec_mult as ExecFn),
            (Mnemonic::DVD, Executor::exec_dvd as ExecFn),
            (Mnemonic::TAKE, Executor::exec_take as ExecFn),
        ]
        .into_iter()
        .collect();

        let f = execution
            .get(&pword.mnemonic())
            .expect("Expected instruction to be implemented");
        f(self, acc, store_operand)
    }

    fn value_at(&self, store_operand: &StoreOperand) -> MemoryWord {
        match store_operand {
            StoreOperand::None => MemoryWord::Undefined,
            StoreOperand::ConstOperand(value) => match value {
                ConstOperand::SignedInteger(i) => MemoryWord::Integer(*i),
                ConstOperand::SignedFWord(f) => MemoryWord::Float(*f),
                ConstOperand::SWord(s) => MemoryWord::String(s.clone()),
            },
            StoreOperand::AddressOperand(address) => {
                let location = self.address_of(&address.address()) + address.index().unwrap_or(0);
                self.execution_context.memory[location].clone()
            }
        }
    }

    fn address_of(&self, address_operand: &SimpleAddressOperand) -> Location {
        match address_operand {
            SimpleAddressOperand::DirectAddress(a) => self.location_for(a),
            SimpleAddressOperand::IndirectAddress(a) => {
                let _content = &self.execution_context.memory[self.location_for(a)];
                // TODO: How to interpret undiection with "type" embedded in MemoryWord...
                unimplemented!()
            }
        }
    }

    fn location_for(&self, address: &Address) -> Location {
        match address {
            Address::NumericAddress(a) => *a,
            Address::Identifier(_) => unreachable!(), // Identifiers linked to actual addresses
        }
    }

    fn exec_nil(&mut self, _acc: Location, _operand: MemoryWord) {}

    fn exec_or(&mut self, acc: Location, operand: MemoryWord) {
        self.execution_context.memory[acc] |= operand;
    }

    fn exec_neqv(&mut self, acc: Location, operand: MemoryWord) {
        self.execution_context.memory[acc] ^= operand;
    }

    fn exec_and(&mut self, acc: Location, operand: MemoryWord) {
        self.execution_context.memory[acc] &= operand;
    }

    fn exec_add(&mut self, acc: Location, operand: MemoryWord) {
        self.execution_context.memory[acc] += operand;
    }

    fn exec_subt(&mut self, acc: Location, operand: MemoryWord) {
        self.execution_context.memory[acc] -= operand;
    }

    fn exec_mult(&mut self, acc: Location, operand: MemoryWord) {
        self.execution_context.memory[acc] *= operand;
    }

    fn exec_dvd(&mut self, acc: Location, operand: MemoryWord) {
        self.execution_context.memory[acc] /= operand;
    }

    fn exec_take(&mut self, acc: Location, operand: MemoryWord) {
        self.execution_context.memory[acc] = operand;
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct ExecutionContext {
    program_counter: Location,
    memory: Memory,
}

impl ExecutionContext {
    #[cfg(test)]
    fn with_program_counter(self, location: Location) -> Self {
        Self {
            program_counter: location,
            ..self
        }
    }

    #[cfg(test)]
    fn with_memory_word(mut self, location: Location, value: MemoryWord) -> Self {
        self.memory[location] = value;
        self
    }

    #[cfg(test)]
    fn with_instruction(self, location: Location, pword: PWord) -> Self {
        self.with_memory_word(location, MemoryWord::Instruction(pword))
    }
}

impl From<Assembly> for ExecutionContext {
    fn from(value: Assembly) -> Self {
        let value = value.allocate_storage_locations();
        let program_counter = value.first_pword_location().unwrap_or(0);
        let memory = Memory::from(value);

        Self {
            program_counter,
            memory,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::bbcx::assembler::*;
    use crate::bbcx::parser::*;

    fn execute(input: &str) -> Result<ExecutionContext> {
        let program = input
            .to_string()
            .lines()
            .map(Parser::parse_line)
            .filter_map(|l| l.ok())
            .collect::<Vec<_>>();
        let assembly = Assembler::assemble(&program)?;
        Executor::execute(assembly)
    }

    #[test]
    fn default_execution_context() {
        let program = r#"
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default().with_program_counter(0);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_nil() {
        let program = r#"
0001    NIL
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_program_counter(2)
            .with_instruction(
                1,
                PWord::new(Mnemonic::NIL, None.into(), StoreOperand::None),
            );
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_or() {
        let program = r#"
0001    +12
0100    OR 1, +10
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                PWord::new(
                    Mnemonic::OR,
                    '1'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedInteger(10)),
                ),
            )
            .with_program_counter(101)
            .with_memory_word(1, MemoryWord::Integer(14));
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_neqv() {
        let program = r#"
0001    +12
0100    NEQV 1, +10
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                PWord::new(
                    Mnemonic::NEQV,
                    '1'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedInteger(10)),
                ),
            )
            .with_program_counter(101)
            .with_memory_word(1, MemoryWord::Integer(6));
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_and() {
        let program = r#"
0001    +12
0100    AND 1, +10
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                PWord::new(
                    Mnemonic::AND,
                    '1'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedInteger(10)),
                ),
            )
            .with_program_counter(101)
            .with_memory_word(1, MemoryWord::Integer(8));
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_add() {
        let program = r#"
0001    +12
0100    ADD 1, +10
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                PWord::new(
                    Mnemonic::ADD,
                    '1'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedInteger(10)),
                ),
            )
            .with_program_counter(101)
            .with_memory_word(1, MemoryWord::Integer(22));
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_subt() {
        let program = r#"
0001    +12
0002    +10
0100    SUBT 1, +10
0101    SUBT 2, +12
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                PWord::new(
                    Mnemonic::SUBT,
                    '1'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedInteger(10)),
                ),
            )
            .with_instruction(
                101,
                PWord::new(
                    Mnemonic::SUBT,
                    '2'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedInteger(12)),
                ),
            )
            .with_program_counter(102)
            .with_memory_word(1, MemoryWord::Integer(2))
            .with_memory_word(2, MemoryWord::Integer(-2));
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_mult() {
        let program = r#"
0001    +12
0100    MULT 1, +10
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                PWord::new(
                    Mnemonic::MULT,
                    '1'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedInteger(10)),
                ),
            )
            .with_program_counter(101)
            .with_memory_word(1, MemoryWord::Integer(120));
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_dvd() {
        let program = r#"
0001    +12
0100    DVD 1, +6
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                PWord::new(
                    Mnemonic::DVD,
                    '1'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedInteger(6)),
                ),
            )
            .with_program_counter(101)
            .with_memory_word(1, MemoryWord::Integer(2));
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_take() {
        let program = r#"
            0100            TAKE 1, +42
            0101            TAKE 2, +3.14
            0102            TAKE 3, "ABCD"
            0103            TAKE 4, LOC
            0110    LOC:    +2.718
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                PWord::new(
                    Mnemonic::TAKE,
                    '1'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedInteger(42)),
                ),
            )
            .with_instruction(
                101,
                PWord::new(
                    Mnemonic::TAKE,
                    '2'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedFWord(3.14)),
                ),
            )
            .with_instruction(
                102,
                PWord::new(
                    Mnemonic::TAKE,
                    '3'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SWord("ABCD".into())),
                ),
            )
            .with_instruction(
                103,
                PWord::new(
                    Mnemonic::TAKE,
                    '4'.into(),
                    StoreOperand::AddressOperand(AddressOperand::new(
                        SimpleAddressOperand::DirectAddress(Address::NumericAddress(110)),
                        None,
                    )),
                ),
            )
            .with_program_counter(104)
            .with_memory_word(1, MemoryWord::Integer(42))
            .with_memory_word(2, MemoryWord::Float(3.14))
            .with_memory_word(3, MemoryWord::String("ABCD".into()))
            .with_memory_word(4, MemoryWord::Float(2.718))
            .with_memory_word(110, MemoryWord::Float(2.718));
        assert_eq!(actual, expected)
    }
}
