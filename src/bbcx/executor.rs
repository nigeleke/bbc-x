use super::assembly::Assembly;
use super::memory::{word_to_instruction, Address, Instruction, MemoryIndex, *};
use super::result::{Error, Result};

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::rc::Rc;

pub struct Executor {
    ec: ExecutionContext,
    stdin: Rc<RefCell<dyn Read>>,
    stdout: Rc<RefCell<dyn Write>>,
}

impl Executor {
    pub fn new() -> Self {
        let stdin = Rc::new(RefCell::new(io::stdin()));
        let stdout = Rc::new(RefCell::new(io::stdout()));
        Self::with_io(stdin, stdout)
    }

    pub fn with_io<R, W>(stdin: Rc<RefCell<R>>, stdout: Rc<RefCell<W>>) -> Self
    where
        R: Read + 'static,
        W: Write + 'static,
    {
        Self {
            ec: ExecutionContext::default(),
            stdin,
            stdout,
        }
    }

    pub fn execute(mut self, assembly: &Assembly) -> Result<ExecutionContext> {
        self.ec = assembly.clone().try_into()?;

        while self.can_step() {
            self.step()?;
        }
        Ok(self.ec.clone())
    }

    fn can_step(&self) -> bool {
        let context = &self.ec;
        let content = &context[context.pc];
        content.is_instruction()
    }

    fn step(&mut self) -> Result<()> {
        let pc = self.ec.pc;
        self.ec.pc += 1;
        let content = self.ec[pc];
        let instruction = word_to_instruction(&content)
            .map_err(|err| Error::CannotConvertWordToInstruction(err.to_string()))?;
        println!("Exec: {:?} -> {:?}", self.ec.pc - 1, instruction);
        self.step_word(&instruction);
        Ok(())
    }

    fn step_word(&mut self, instruction: &Instruction) {
        type ExecFn = fn(&mut Executor, &Instruction);
        let execution: HashMap<Function, ExecFn> = vec![
            (Function::NIL, Executor::exec_nil as ExecFn),
            (Function::OR, Executor::exec_or as ExecFn),
            (Function::NEQV, Executor::exec_neqv as ExecFn),
            (Function::AND, Executor::exec_and as ExecFn),
            (Function::ADD, Executor::exec_add as ExecFn),
            (Function::SUBT, Executor::exec_subt as ExecFn),
            (Function::MULT, Executor::exec_mult as ExecFn),
            (Function::DVD, Executor::exec_dvd as ExecFn),
            (Function::TAKE, Executor::exec_take as ExecFn),
            (Function::TSTR, Executor::exec_tstr as ExecFn),
            (Function::TNEG, Executor::exec_tneg as ExecFn),
            (Function::TNOT, Executor::exec_tnot as ExecFn),
            (Function::TTYP, Executor::exec_ttyp as ExecFn),
            (Function::TTYZ, Executor::exec_ttyz as ExecFn),
            (Function::TOUT, Executor::exec_tout as ExecFn),
            (Function::SKIP, Executor::exec_skip as ExecFn),
            (Function::SKAE, Executor::exec_skae as ExecFn),
            (Function::SKAN, Executor::exec_skan as ExecFn),
            (Function::SKET, Executor::exec_sket as ExecFn),
            (Function::SKAL, Executor::exec_skal as ExecFn),
            (Function::SKAG, Executor::exec_skag as ExecFn),
            (Function::SKED, Executor::exec_sked as ExecFn),
            (Function::SKEI, Executor::exec_skei as ExecFn),
            (Function::SHL, Executor::exec_shl as ExecFn),
            (Function::ROT, Executor::exec_rot as ExecFn),
            (Function::DSHL, Executor::exec_dshl as ExecFn),
            (Function::DROT, Executor::exec_drot as ExecFn),
            (Function::POWR, Executor::exec_powr as ExecFn),
            (Function::DMULT, Executor::exec_dmult as ExecFn),
            (Function::DIV, Executor::exec_div as ExecFn),
            (Function::DDIV, Executor::exec_ddiv as ExecFn),
            (Function::NILX, Executor::exec_nilx as ExecFn),
            (Function::ORX, Executor::exec_orx as ExecFn),
            (Function::NEQVX, Executor::exec_neqvx as ExecFn),
            (Function::ADDX, Executor::exec_addx as ExecFn),
            (Function::SUBTX, Executor::exec_subtx as ExecFn),
            (Function::MULTX, Executor::exec_multx as ExecFn),
            (Function::DVDX, Executor::exec_dvdx as ExecFn),
            (Function::PUT, Executor::exec_put as ExecFn),
            (Function::PSQU, Executor::exec_psqu as ExecFn),
            (Function::PNEG, Executor::exec_pneg as ExecFn),
            (Function::PTYP, Executor::exec_ptyp as ExecFn),
            (Function::PTYZ, Executor::exec_ptyz as ExecFn),
            (Function::PIN, Executor::exec_pin as ExecFn),
            (Function::JUMP, Executor::exec_jump as ExecFn),
        ]
        .into_iter()
        .collect();

        let f = execution
            .get(&instruction.function())
            .expect("Expected instruction to be implemented");
        f(self, instruction)
    }

    fn operand(&self, instruction: &Instruction) -> Result<Word> {
        let ec = &self.ec;

        let index_register = instruction.index_register();
        let mut address = instruction.address();

        if instruction.is_indirect() {
            let indirect_instruction = word_to_instruction(&ec[address])
                .map_err(|err| Error::CannotDetermineOperand(err.to_string()))?;
            address = indirect_instruction.address()
        }

        if index_register.is_indexable() {
            let index = ec[index_register]
                .as_i64()
                .map_err(|err| Error::CannotDetermineOperand(err.to_string()))?;
            address += index as isize;
        }

        Ok(ec[address])
    }

    fn acc_and_operand(&self, instruction: &Instruction) -> (Accumulator, Word) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction).expect("Invalid operand");
        (acc, operand)
    }

    fn acc_and_address(&instruction: &Instruction) -> (Accumulator, Address) {
        let acc = instruction.accumulator();
        let address = instruction.address();
        (acc, address)
    }

    fn exec_nil(&mut self, _instruction: &Instruction) {}

    fn exec_or(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc] |= operand;
    }

    fn exec_neqv(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc] ^= operand;
    }

    fn exec_and(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc] &= operand;
    }

    fn exec_add(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc] += operand;
    }

    fn exec_subt(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc] -= operand;
    }

    fn exec_mult(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc] *= operand;
    }

    fn exec_dvd(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc] /= operand;
    }

    fn exec_take(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc] = operand;
    }

    fn exec_tstr(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        let value = operand.as_i64().expect("TSTR invalid operand");
        self.ec[acc] = operand;
        self.ec[acc - 1] = (if value < 1 { -1 } else { 0 }).try_into().unwrap();
    }

    fn exec_tneg(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc] = -operand;
    }

    fn exec_tnot(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc] = !operand;
    }

    fn exec_ttyp(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc] = operand.word_type();
    }

    fn exec_ttyz(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc] = operand.word_bits();
    }

    fn exec_tout(&mut self, instruction: &Instruction) {
        let (_acc, operand) = self.acc_and_operand(instruction);
        let chars = vec![operand.as_char().expect("TOUT invalid operand")];
        let mut stdout = (*self.stdout).borrow_mut();
        stdout.write_all(&chars).unwrap();
    }

    fn exec_skip(&mut self, _instruction: &Instruction) {
        self.ec.pc += 1;
    }

    fn exec_skae(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        if self.ec[acc] == operand {
            self.ec.pc += 1;
        }
    }

    fn exec_skan(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        if self.ec[acc] != operand {
            self.ec.pc += 1;
        }
    }

    fn exec_sket(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        let same_type = self.ec[acc].word_type() == operand.word_type();
        if same_type {
            self.ec.pc += 1;
        }
    }

    fn exec_skal(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        if self.ec[acc] < operand {
            self.ec.pc += 1
        }
    }

    fn exec_skag(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        if self.ec[acc] > operand {
            self.ec.pc += 1
        }
    }

    fn exec_sked(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        if self.ec[acc] == operand {
            self.ec.pc += 1
        } else {
            self.ec[acc] -= 1.try_into().unwrap()
        }
    }

    fn exec_skei(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        if self.ec[acc] == operand {
            self.ec.pc += 1
        } else {
            self.ec[acc] += 1.try_into().unwrap()
        }
    }

    fn exec_shl(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc] <<= operand;
    }

    fn exec_rot(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc].rotate(operand.as_i64().unwrap());
    }

    fn exec_dshl(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        let (msw, lsw) = double_shift_left(&self.ec[acc - 1], &self.ec[acc], &operand).unwrap();
        self.ec[acc - 1].set_word_bits(&msw);
        self.ec[acc].set_word_bits(&lsw);
    }

    fn exec_drot(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        let (msw, lsw) = double_rotate_left(&self.ec[acc - 1], &self.ec[acc], &operand).unwrap();
        self.ec[acc - 1].set_word_bits(&msw);
        self.ec[acc].set_word_bits(&lsw);
    }

    fn exec_powr(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc].power(&operand);
    }

    fn exec_dmult(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        let (msw, lsw) = double_mult(&self.ec[acc - 1], &self.ec[acc], &operand).unwrap();
        self.ec[acc - 1].set_word_bits(&msw);
        self.ec[acc].set_word_bits(&lsw);
    }

    fn exec_div(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        self.ec[acc] /= operand;
    }

    fn exec_ddiv(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        let (msw, lsw) = double_div(&self.ec[acc - 1], &self.ec[acc], &operand).unwrap();
        self.ec[acc - 1].set_word_bits(&msw);
        self.ec[acc].set_word_bits(&lsw);
    }

    fn exec_nilx(&mut self, instruction: &Instruction) {
        let (acc, operand) = self.acc_and_operand(instruction);
        let acc_value = self.ec[acc];
        let operand_address = instruction.address();
        self.ec[acc] = operand;
        self.ec[operand_address] = acc_value;
    }

    fn exec_orx(&mut self, instruction: &Instruction) {
        self.exec_or(instruction);
        self.exec_nilx(instruction);
    }

    fn exec_neqvx(&mut self, instruction: &Instruction) {
        self.exec_neqv(instruction);
        self.exec_nilx(instruction);
    }

    fn exec_addx(&mut self, instruction: &Instruction) {
        self.exec_add(instruction);
        self.exec_nilx(instruction);
    }

    fn exec_subtx(&mut self, instruction: &Instruction) {
        self.exec_subt(instruction);
        self.exec_nilx(instruction);
    }

    fn exec_multx(&mut self, instruction: &Instruction) {
        self.exec_mult(instruction);
        self.exec_nilx(instruction);
    }

    fn exec_dvdx(&mut self, instruction: &Instruction) {
        self.exec_dvd(instruction);
        self.exec_nilx(instruction);
    }

    fn exec_put(&mut self, instruction: &Instruction) {
        let (acc, address) = Self::acc_and_address(instruction);
        let acc_value = self.ec[acc];
        self.ec[address] = acc_value;
    }

    fn exec_psqu(&mut self, instruction: &Instruction) {
        let (acc, address) = Self::acc_and_address(instruction);
        let msw0 = self.ec[acc - 1];
        let lsw0 = self.ec[acc];
        let (_, mut lsw1) = squash(&msw0, &lsw0).unwrap();
        lsw1.set_word_type(&lsw0);
        self.ec[address] = lsw1;
    }

    fn exec_pneg(&mut self, instruction: &Instruction) {
        let (acc, address) = Self::acc_and_address(instruction);
        let acc_value = self.ec[acc];
        self.ec[address] = -acc_value;
    }

    fn exec_ptyp(&mut self, instruction: &Instruction) {
        let (acc, address) = Self::acc_and_address(instruction);
        let acc_value = self.ec[acc];
        self.ec[address].set_word_type(&acc_value);
    }

    fn exec_ptyz(&mut self, instruction: &Instruction) {
        let (acc, address) = Self::acc_and_address(instruction);
        let acc_value = self.ec[acc];
        let mut result: Word = 0.try_into().unwrap();
        result.set_word_bits(&acc_value);
        self.ec[address] = result;
    }

    fn exec_pin(&mut self, instruction: &Instruction) {
        let (_acc, address) = Self::acc_and_address(instruction);

        let mut stdin = (*self.stdin).borrow_mut();
        let mut stdout = (*self.stdout).borrow_mut();

        let mut buffer = vec![0u8; 1];

        match stdin.read(&mut buffer) {
            Ok(0) => {
                stdout.write_all("DATA*".as_bytes()).unwrap();
            }
            Ok(_) => {
                // TODO: May not want to echo here, but just set `self.ec[address]`
                stdout.write_all(&buffer).unwrap();
                self.ec[address] = String::from_utf8(buffer)
                    .map(|s| s.as_str().try_into().unwrap())
                    .unwrap();
            }
            Err(e) => {
                panic!("Error reading from stdin: {}", e);
            }
        }
    }

    fn exec_jump(&mut self, instruction: &Instruction) {
        let pc = self.ec.pc - 1;
        let (acc, address) = Self::acc_and_address(instruction);
        self.ec[acc - 1] = (pc.memory_index() as i64).try_into().unwrap();
        self.ec.pc = address;
    }
}

impl std::fmt::Debug for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Executor")
            .field("execution_context", &self.ec)
            .field("stdin", &"Box<std::io::Read>")
            .field("stdout", &"Box<std::io::Write>")
            .finish()
    }
}

impl PartialEq for Executor {
    fn eq(&self, other: &Self) -> bool {
        self.ec.eq(&other.ec)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExecutionContext {
    pc: Address,
    memory: Memory,
}

#[cfg(test)]
impl ExecutionContext {
    fn with_program_counter<A>(self, program_counter: A) -> Self
    where
        A: TryInto<Address>,
        A::Error: std::fmt::Debug,
    {
        Self {
            pc: program_counter.try_into().unwrap(),
            ..self
        }
    }

    fn with_memory_word<A, T>(mut self, address: A, value: T) -> Self
    where
        A: TryInto<Address>,
        A::Error: std::fmt::Debug,
        T: TryInto<Word>,
        T::Error: std::fmt::Debug,
    {
        let address = address.try_into().unwrap();
        self.memory[address.memory_index()] = value
            .try_into()
            .expect("required valid value to create word");
        self
    }

    fn with_instruction<A>(self, address: A, instruction: Instruction) -> Self
    where
        A: TryInto<Address>,
        A::Error: std::fmt::Debug,
    {
        use crate::bbcx::memory::*;
        self.with_memory_word(address, instruction_to_word(&instruction).unwrap())
    }
}

impl TryFrom<Assembly> for ExecutionContext {
    type Error = Error;

    fn try_from(value: Assembly) -> std::result::Result<Self, Self::Error> {
        let value = value.allocate_storage_locations();
        let program_counter = value.first_pword_location().unwrap_or(0);
        let memory = Memory::try_from(value)
            .map_err(|err| Error::FailedToCreateExecutionContext(err.to_string()))?;

        Ok(Self {
            pc: program_counter.try_into().unwrap(),
            memory,
        })
    }
}

impl std::ops::Index<usize> for ExecutionContext {
    type Output = Word;

    fn index(&self, index: usize) -> &Self::Output {
        &self.memory[index]
    }
}

impl std::ops::IndexMut<usize> for ExecutionContext {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.memory[index]
    }
}

impl std::ops::Index<Accumulator> for ExecutionContext {
    type Output = Word;

    fn index(&self, acc: Accumulator) -> &Self::Output {
        &self.memory[acc]
    }
}

impl std::ops::IndexMut<Accumulator> for ExecutionContext {
    fn index_mut(&mut self, acc: Accumulator) -> &mut Self::Output {
        &mut self.memory[acc]
    }
}

impl std::ops::Index<IndexRegister> for ExecutionContext {
    type Output = Word;

    fn index(&self, index_register: IndexRegister) -> &Self::Output {
        &self.memory[index_register]
    }
}

impl std::ops::IndexMut<IndexRegister> for ExecutionContext {
    fn index_mut(&mut self, index_register: IndexRegister) -> &mut Self::Output {
        &mut self.memory[index_register]
    }
}

impl std::ops::Index<Address> for ExecutionContext {
    type Output = Word;

    fn index(&self, address: Address) -> &Self::Output {
        &self.memory[address]
    }
}

impl std::ops::IndexMut<Address> for ExecutionContext {
    fn index_mut(&mut self, address: Address) -> &mut Self::Output {
        &mut self.memory[address]
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use std::io::Cursor;

    use super::*;

    use crate::bbcx::assembler::*;
    use crate::bbcx::parser::*;

    fn execute(input: &str) -> Result<ExecutionContext> {
        let executor = Executor::new();
        do_execute(input, executor)
    }

    fn execute_io(input: &str, stdin: &str, expected: &str) -> Result<ExecutionContext> {
        let stdin = String::from(stdin);
        let stdin_buffer = Cursor::new(stdin);
        let stdin = Rc::new(RefCell::new(std::io::BufReader::new(stdin_buffer)));

        let stdout_buffer = Vec::new();
        let stdout = Rc::new(RefCell::new(std::io::BufWriter::new(stdout_buffer)));

        let executor = Executor::with_io(stdin, stdout.clone());
        let ec = do_execute(input, executor).unwrap().clone();

        let stdout = stdout.borrow();
        let bytes = stdout.buffer();
        let actual = String::from_utf8_lossy(&bytes);

        assert_eq!(actual, String::from(expected));

        Ok(ec)
    }

    fn do_execute(input: &str, executor: Executor) -> Result<ExecutionContext> {
        let program = input
            .to_string()
            .lines()
            .map(Parser::parse_line)
            .filter_map(|l| l.ok())
            .collect::<Vec<_>>();
        let assembly =
            Assembler::assemble(&program).expect(&format!("Failed to assemble {}", input));
        let ec = executor
            .execute(&assembly)
            .expect(&format!("Failed to execute {}", input));
        Ok(ec.clone())
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
0001    +3.14
0100    NIL 1, +2.71
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::NIL)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, 3.14)
            .with_memory_word(MEMORY_SIZE - 1, 2.71)
            .with_program_counter(101);
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
                InstructionBuilder::new(Function::OR)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, 14)
            .with_memory_word(MEMORY_SIZE - 1, 10)
            .with_program_counter(101);
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
                InstructionBuilder::new(Function::NEQV)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, 6)
            .with_memory_word(MEMORY_SIZE - 1, 10)
            .with_program_counter(101);
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
                InstructionBuilder::new(Function::AND)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, 8)
            .with_memory_word(MEMORY_SIZE - 1, 10)
            .with_program_counter(101);
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
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, 22)
            .with_memory_word(MEMORY_SIZE - 1, 10)
            .with_program_counter(101);
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
                InstructionBuilder::new(Function::SUBT)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::SUBT)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_memory_word(1, 2)
            .with_memory_word(2, -2)
            .with_memory_word(MEMORY_SIZE - 1, 10)
            .with_memory_word(MEMORY_SIZE - 2, 12)
            .with_program_counter(102);
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
                InstructionBuilder::new(Function::MULT)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, 120)
            .with_memory_word(MEMORY_SIZE - 1, 10)
            .with_program_counter(101);
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
                InstructionBuilder::new(Function::DVD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, 2)
            .with_memory_word(MEMORY_SIZE - 1, 6)
            .with_program_counter(101);
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
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_instruction(
                102,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 3)
                    .build(),
            )
            .with_instruction(
                103,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(4)
                    .with_address(110)
                    .build(),
            )
            .with_memory_word(1, 42)
            .with_memory_word(2, 3.14)
            .with_memory_word(3, "ABCD")
            .with_memory_word(4, 2.718)
            .with_memory_word(MEMORY_SIZE - 1, 42)
            .with_memory_word(MEMORY_SIZE - 2, 3.14)
            .with_memory_word(MEMORY_SIZE - 3, "ABCD")
            .with_memory_word(110, 2.718)
            .with_program_counter(104);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_tstr() {
        let program = r#"
0100    TSTR 2, +2
0101    TSTR 4, -2
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::TSTR)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::TSTR)
                    .with_accumulator(4)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_memory_word(1, 0)
            .with_memory_word(2, 2)
            .with_memory_word(3, -1)
            .with_memory_word(4, -2)
            .with_memory_word(MEMORY_SIZE - 1, 2)
            .with_memory_word(MEMORY_SIZE - 2, -2)
            .with_program_counter(102);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_tneg() {
        let program = r#"
0100    TNEG 1, +6
0101    TNEG 2, -6
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::TNEG)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::TNEG)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_memory_word(1, -6)
            .with_memory_word(2, 6)
            .with_memory_word(MEMORY_SIZE - 1, 6)
            .with_memory_word(MEMORY_SIZE - 2, -6)
            .with_program_counter(102);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_tnot() {
        let program = r#"
0100    TNOT 1, +5592405
0101    TNOT 2, -5592406
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::TNOT)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::TNOT)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_memory_word(1, -5592406)
            .with_memory_word(2, 5592405)
            .with_memory_word(MEMORY_SIZE - 1, 5592405)
            .with_memory_word(MEMORY_SIZE - 2, -5592406)
            .with_program_counter(102);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_ttyp() {
        let program = r#"
0100    TTYP 1, +42
0101    TTYP 2, +3.14
0102    TTYP 3, "ABCD"
0103    TTYP 4, 103
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::TTYP)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::TTYP)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_instruction(
                102,
                InstructionBuilder::new(Function::TTYP)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 3)
                    .build(),
            )
            .with_instruction(
                103,
                InstructionBuilder::new(Function::TTYP)
                    .with_accumulator(4)
                    .with_address(103)
                    .build(),
            )
            .with_memory_word(1, 0)
            .with_memory_word(2, 1)
            .with_memory_word(3, 2)
            .with_memory_word(4, 3)
            .with_memory_word(MEMORY_SIZE - 1, 42)
            .with_memory_word(MEMORY_SIZE - 2, 3.14)
            .with_memory_word(MEMORY_SIZE - 3, "ABCD")
            .with_program_counter(104);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_ttyz() {
        let program = r#"
0100    TTYZ 1, +42
0101    TTYZ 2, "ABCD"
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::TTYZ)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::TTYZ)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_memory_word(1, 42)
            .with_memory_word(2, 0o01020304)
            .with_memory_word(MEMORY_SIZE - 1, 42)
            .with_memory_word(MEMORY_SIZE - 2, "ABCD")
            .with_program_counter(102);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_tttt() {
        // Spec: "not used at present"
        assert!(true)
    }

    #[test]
    fn test_tout() {
        let program = r#"
0100    TOUT +1
0101    TOUT "ABCD"
"#;
        let actual = execute_io(program, "", "AD").ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::TOUT)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::TOUT)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_memory_word(MEMORY_SIZE - 1, 1)
            .with_memory_word(MEMORY_SIZE - 2, "ABCD")
            .with_program_counter(102);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_skip() {
        let program = r#"
0001    +0
0100    ADD 1, +1
0101    SKIP
0102    ADD 1, +1
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(101, InstructionBuilder::new(Function::SKIP).build())
            .with_instruction(
                102,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_memory_word(1, 1)
            .with_memory_word(MEMORY_SIZE - 1, 1)
            .with_memory_word(MEMORY_SIZE - 2, 1)
            .with_program_counter(103);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_skae() {
        let program = r#"
0001    +0
0002    +0
0100    ADD 1, +1
0101    SKAE 1, +1
0102    ADD 1, +1
0103    ADD 2, +1
0104    SKAE 2, +2
0105    ADD 2, +1
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::SKAE)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_instruction(
                102,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 3)
                    .build(),
            )
            .with_instruction(
                103,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 4)
                    .build(),
            )
            .with_instruction(
                104,
                InstructionBuilder::new(Function::SKAE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 5)
                    .build(),
            )
            .with_instruction(
                105,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 6)
                    .build(),
            )
            .with_memory_word(1, 1)
            .with_memory_word(2, 2)
            .with_memory_word(MEMORY_SIZE - 1, 1)
            .with_memory_word(MEMORY_SIZE - 2, 1)
            .with_memory_word(MEMORY_SIZE - 3, 1)
            .with_memory_word(MEMORY_SIZE - 4, 1)
            .with_memory_word(MEMORY_SIZE - 5, 2)
            .with_memory_word(MEMORY_SIZE - 6, 1)
            .with_program_counter(106);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_skan() {
        let program = r#"
0001    +0
0002    +0
0100    ADD 1, +1
0101    SKAN 1, +1
0102    ADD 1, +1
0103    ADD 2, +1
0104    SKAN 2, +2
0105    ADD 2, +1
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::SKAN)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_instruction(
                102,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 3)
                    .build(),
            )
            .with_instruction(
                103,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 4)
                    .build(),
            )
            .with_instruction(
                104,
                InstructionBuilder::new(Function::SKAN)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 5)
                    .build(),
            )
            .with_instruction(
                105,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 6)
                    .build(),
            )
            .with_memory_word(1, 2)
            .with_memory_word(2, 1)
            .with_memory_word(MEMORY_SIZE - 1, 1)
            .with_memory_word(MEMORY_SIZE - 2, 1)
            .with_memory_word(MEMORY_SIZE - 3, 1)
            .with_memory_word(MEMORY_SIZE - 4, 1)
            .with_memory_word(MEMORY_SIZE - 5, 2)
            .with_memory_word(MEMORY_SIZE - 6, 1)
            .with_program_counter(106);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_sket() {
        let program = r#"
0100    TAKE 1, +1.0
0101    SKET 1, +1.0
0102    ADD 1, +1.0
0103    TAKE 2, +1.0
0104    SKET 2, +1
0105    ADD 2, +1.0
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::SKET)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_instruction(
                102,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 3)
                    .build(),
            )
            .with_instruction(
                103,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 4)
                    .build(),
            )
            .with_instruction(
                104,
                InstructionBuilder::new(Function::SKET)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 5)
                    .build(),
            )
            .with_instruction(
                105,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 6)
                    .build(),
            )
            .with_memory_word(1, 1.0)
            .with_memory_word(2, 2.0)
            .with_memory_word(MEMORY_SIZE - 1, 1.0)
            .with_memory_word(MEMORY_SIZE - 2, 1.0)
            .with_memory_word(MEMORY_SIZE - 3, 1.0)
            .with_memory_word(MEMORY_SIZE - 4, 1.0)
            .with_memory_word(MEMORY_SIZE - 5, 1)
            .with_memory_word(MEMORY_SIZE - 6, 1.0)
            .with_program_counter(106);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_skal() {
        let program = r#"
0100    TAKE 1, +0.0
0101    SKAL 1, +1.0
0102    ADD 1, +1.0
0103    TAKE 2, +1.0
0104    SKAL 2, +1.0
0105    ADD 2, +1.0
0106    TAKE 3, +2.0
0107    SKAL 3, +1.0
0108    ADD 3, +1.0
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::SKAL)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_instruction(
                102,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 3)
                    .build(),
            )
            .with_instruction(
                103,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 4)
                    .build(),
            )
            .with_instruction(
                104,
                InstructionBuilder::new(Function::SKAL)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 5)
                    .build(),
            )
            .with_instruction(
                105,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 6)
                    .build(),
            )
            .with_instruction(
                106,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 7)
                    .build(),
            )
            .with_instruction(
                107,
                InstructionBuilder::new(Function::SKAL)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 8)
                    .build(),
            )
            .with_instruction(
                108,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 9)
                    .build(),
            )
            .with_memory_word(1, 0.0)
            .with_memory_word(2, 2.0)
            .with_memory_word(3, 3.0)
            .with_memory_word(MEMORY_SIZE - 1, 0.0)
            .with_memory_word(MEMORY_SIZE - 2, 1.0)
            .with_memory_word(MEMORY_SIZE - 3, 1.0)
            .with_memory_word(MEMORY_SIZE - 4, 1.0)
            .with_memory_word(MEMORY_SIZE - 5, 1.0)
            .with_memory_word(MEMORY_SIZE - 6, 1.0)
            .with_memory_word(MEMORY_SIZE - 7, 2.0)
            .with_memory_word(MEMORY_SIZE - 8, 1.0)
            .with_memory_word(MEMORY_SIZE - 9, 1.0)
            .with_program_counter(109);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_skag() {
        let program = r#"
0100    TAKE 1, +0.0
0101    SKAG 1, +1.0
0102    ADD 1, +1.0
0103    TAKE 2, +1.0
0104    SKAG 2, +1.0
0105    ADD 2, +1.0
0106    TAKE 3, +2.0
0107    SKAG 3, +1.0
0108    ADD 3, +1.0
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::SKAG)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_instruction(
                102,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 3)
                    .build(),
            )
            .with_instruction(
                103,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 4)
                    .build(),
            )
            .with_instruction(
                104,
                InstructionBuilder::new(Function::SKAG)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 5)
                    .build(),
            )
            .with_instruction(
                105,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 6)
                    .build(),
            )
            .with_instruction(
                106,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 7)
                    .build(),
            )
            .with_instruction(
                107,
                InstructionBuilder::new(Function::SKAG)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 8)
                    .build(),
            )
            .with_instruction(
                108,
                InstructionBuilder::new(Function::ADD)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 9)
                    .build(),
            )
            .with_memory_word(1, 1.0)
            .with_memory_word(2, 2.0)
            .with_memory_word(3, 2.0)
            .with_memory_word(MEMORY_SIZE - 1, 0.0)
            .with_memory_word(MEMORY_SIZE - 2, 1.0)
            .with_memory_word(MEMORY_SIZE - 3, 1.0)
            .with_memory_word(MEMORY_SIZE - 4, 1.0)
            .with_memory_word(MEMORY_SIZE - 5, 1.0)
            .with_memory_word(MEMORY_SIZE - 6, 1.0)
            .with_memory_word(MEMORY_SIZE - 7, 2.0)
            .with_memory_word(MEMORY_SIZE - 8, 1.0)
            .with_memory_word(MEMORY_SIZE - 9, 1.0)
            .with_program_counter(109);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_sked() {
        let program = r#"
0100    TAKE 1, +42
0101    SKED 1, +42
0102    NIL
0103    TAKE 2, +1
0104    SKED 2, +42
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::SKED)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_instruction(102, InstructionBuilder::new(Function::NIL).build())
            .with_instruction(
                103,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 3)
                    .build(),
            )
            .with_instruction(
                104,
                InstructionBuilder::new(Function::SKED)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 4)
                    .build(),
            )
            .with_memory_word(1, 42)
            .with_memory_word(2, 0)
            .with_memory_word(MEMORY_SIZE - 1, 42)
            .with_memory_word(MEMORY_SIZE - 2, 42)
            .with_memory_word(MEMORY_SIZE - 3, 1)
            .with_memory_word(MEMORY_SIZE - 4, 42)
            .with_program_counter(105);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_skei() {
        let program = r#"
0100    TAKE 1, +42
0101    SKEI 1, +42
0102    NIL
0103    TAKE 2, +1
0104    SKEI 2, +42
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::SKEI)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_instruction(102, InstructionBuilder::new(Function::NIL).build())
            .with_instruction(
                103,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 3)
                    .build(),
            )
            .with_instruction(
                104,
                InstructionBuilder::new(Function::SKEI)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 4)
                    .build(),
            )
            .with_memory_word(1, 42)
            .with_memory_word(2, 2)
            .with_memory_word(MEMORY_SIZE - 1, 42)
            .with_memory_word(MEMORY_SIZE - 2, 42)
            .with_memory_word(MEMORY_SIZE - 3, 1)
            .with_memory_word(MEMORY_SIZE - 4, 42)
            .with_program_counter(105);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_shl() {
        let program = r#"
0001    +1
0002    +1.0
0003    "  AB"
0100    SHL 1, +1
0101    SHL 2, +1
0102    SHL 3, +6
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::SHL)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::SHL)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_instruction(
                102,
                InstructionBuilder::new(Function::SHL)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 3)
                    .build(),
            )
            .with_memory_word(1, 2)
            .with_memory_word(2, 9_223_372_036_854_775_808.0)
            .with_memory_word(3, " AB\0")
            .with_memory_word(MEMORY_SIZE - 1, 1)
            .with_memory_word(MEMORY_SIZE - 2, 1)
            .with_memory_word(MEMORY_SIZE - 3, 6)
            .with_program_counter(103);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_rot() {
        let program = r#"
0001    +1
0002    +1.0
0003    "ABCD"
0100    ROT 1, +25
0101    ROT 2, +1
0102    ROT 3, +6
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::ROT)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::ROT)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_instruction(
                102,
                InstructionBuilder::new(Function::ROT)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 3)
                    .build(),
            )
            .with_memory_word(1, 2)
            .with_memory_word(2, 9_223_372_036_854_775_808.0)
            .with_memory_word(3, "BCDA")
            .with_memory_word(MEMORY_SIZE - 1, 25)
            .with_memory_word(MEMORY_SIZE - 2, 1)
            .with_memory_word(MEMORY_SIZE - 3, 6)
            .with_program_counter(103);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_dshl() {
        let program = r#"
0001    +0
0002    +1
0003    "    "
0004    "ABCD"
0100    DSHL 2, +25
0101    DSHL 4, +12
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::DSHL)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::DSHL)
                    .with_accumulator(4)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_memory_word(1, 2)
            .with_memory_word(2, 0)
            .with_memory_word(3, "  AB")
            .with_memory_word(4, "CD\0\0")
            .with_memory_word(MEMORY_SIZE - 1, 25)
            .with_memory_word(MEMORY_SIZE - 2, 12)
            .with_program_counter(102);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_drot() {
        let program = r#"
0001    +2
0002    +1
0003    "WXYZ"
0004    "ABCD"
0100    DROT 2, +25
0101    DROT 4, +12
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::DROT)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::DROT)
                    .with_accumulator(4)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_memory_word(1, 2)
            .with_memory_word(2, 4)
            .with_memory_word(3, "YZAB")
            .with_memory_word(4, "CDWX")
            .with_memory_word(MEMORY_SIZE - 1, 25)
            .with_memory_word(MEMORY_SIZE - 2, 12)
            .with_program_counter(102);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_powr() {
        let program = r#"
0001    +2
0002    +2.0
0100    POWR 1, +3
0101    POWR 2, +3.0
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::POWR)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::POWR)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_memory_word(1, 8)
            .with_memory_word(2, 8.0)
            .with_memory_word(MEMORY_SIZE - 1, 3)
            .with_memory_word(MEMORY_SIZE - 2, 3.0)
            .with_program_counter(102);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_dmult() {
        let program = r#"
0001    -1
0002    -16000
0100    DMULT 2, +12000
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::DMULT)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, -12)
            .with_memory_word(2, -7450624)
            .with_memory_word(MEMORY_SIZE - 1, 12000)
            .with_program_counter(101);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_div() {
        let program = r#"
0001    -42
0002    +41
0100    DIV 1, +7
0101    DIV 2, +7
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::DIV)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::DIV)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_memory_word(1, -6)
            .with_memory_word(2, 5)
            .with_memory_word(MEMORY_SIZE - 1, 7)
            .with_memory_word(MEMORY_SIZE - 2, 7)
            .with_program_counter(102);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_ddiv() {
        let program = r#"
0001    -12
0002    -7450624
0100    DDIV 2, -12000
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::DDIV)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, 0)
            .with_memory_word(2, 16000)
            .with_memory_word(MEMORY_SIZE - 1, -12000)
            .with_program_counter(101);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_nilx() {
        let program = r#"
0001    +3.14
0100    NILX 1, +2.71
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::NILX)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, 2.71)
            .with_memory_word(MEMORY_SIZE - 1, 3.14)
            .with_program_counter(101);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_orx() {
        let program = r#"
0001    +12
0100    ORX 1, +10
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::ORX)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, 10)
            .with_memory_word(MEMORY_SIZE - 1, 14)
            .with_program_counter(101);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_neqvx() {
        let program = r#"
0001    +12
0100    NEQVX 1, +10
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::NEQVX)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, 10)
            .with_memory_word(MEMORY_SIZE - 1, 6)
            .with_program_counter(101);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_addx() {
        let program = r#"
0001    +12
0100    ADDX 1, +10
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::ADDX)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, 10)
            .with_memory_word(MEMORY_SIZE - 1, 22)
            .with_program_counter(101);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_subtx() {
        let program = r#"
0001    +12
0002    +10
0100    SUBTX 1, +10
0101    SUBTX 2, +12
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::SUBTX)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::SUBTX)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_memory_word(1, 10)
            .with_memory_word(2, 12)
            .with_memory_word(MEMORY_SIZE - 1, 2)
            .with_memory_word(MEMORY_SIZE - 2, -2)
            .with_program_counter(102);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_multx() {
        let program = r#"
0001    +12
0100    MULTX 1, +10
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::MULTX)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, 10)
            .with_memory_word(MEMORY_SIZE - 1, 120)
            .with_program_counter(101);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_dvdx() {
        let program = r#"
0001    +12
0100    DVDX 1, +6
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::DVDX)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_memory_word(1, 6)
            .with_memory_word(MEMORY_SIZE - 1, 2)
            .with_program_counter(101);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_put() {
        let program = r#"
0001            +42
0002            +3.14
0003            "ABCD"
0004            +2.718
0100            PUT 1, +0
0101            PUT 2, +0.0
0102            PUT 3, "    "
0103            PUT 4, LOC
0110    LOC:    +0.0
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::PUT)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::PUT)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_instruction(
                102,
                InstructionBuilder::new(Function::PUT)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 3)
                    .build(),
            )
            .with_instruction(
                103,
                InstructionBuilder::new(Function::PUT)
                    .with_accumulator(4)
                    .with_address(110)
                    .build(),
            )
            .with_memory_word(1, 42)
            .with_memory_word(2, 3.14)
            .with_memory_word(3, "ABCD")
            .with_memory_word(4, 2.718)
            .with_memory_word(MEMORY_SIZE - 1, 42)
            .with_memory_word(MEMORY_SIZE - 2, 3.14)
            .with_memory_word(MEMORY_SIZE - 3, "ABCD")
            .with_memory_word(110, 2.718)
            .with_program_counter(104);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_psqu() {
        let program = r#"
0001            -1
0002            -12
0100            PSQU 2, LOC
0110    LOC:    +0
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::PSQU)
                    .with_accumulator(2)
                    .with_address(110)
                    .build(),
            )
            .with_memory_word(1, -1)
            .with_memory_word(2, -12)
            .with_memory_word(110, -12)
            .with_program_counter(101);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_pneg() {
        let program = r#"
0001            +3
0002            -3.14
0100            PNEG 1, LOC1
0101            PNEG 2, LOC2
0110    LOC1:   +0
0111    LOC2:   +0
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::PNEG)
                    .with_accumulator(1)
                    .with_address(110)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::PNEG)
                    .with_accumulator(2)
                    .with_address(111)
                    .build(),
            )
            .with_memory_word(1, 3)
            .with_memory_word(2, -3.14)
            .with_memory_word(110, -3)
            .with_memory_word(111, 3.14)
            .with_program_counter(102);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_ptyz() {
        let program = r#"
0001    +1
0002    +1.0
0003    "ABCD"
0004    +0
0100            PTYZ 1, LOC1
0101            PTYZ 2, LOC2
0102            PTYZ 3, LOC3
0103            TAKE 4, 103
0104            PTYZ 4, LOC4
0110    LOC1:   +0.0
0111    LOC2:   +0.0
0112    LOC3:   +0.0
0113    LOC4:   +0.0
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::PTYZ)
                    .with_accumulator(1)
                    .with_address(110)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::PTYZ)
                    .with_accumulator(2)
                    .with_address(111)
                    .build(),
            )
            .with_instruction(
                102,
                InstructionBuilder::new(Function::PTYZ)
                    .with_accumulator(3)
                    .with_address(112)
                    .build(),
            )
            .with_instruction(
                103,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(4)
                    .with_address(103)
                    .build(),
            )
            .with_instruction(
                104,
                InstructionBuilder::new(Function::PTYZ)
                    .with_accumulator(4)
                    .with_address(113)
                    .build(),
            )
            .with_memory_word(1, 1)
            .with_memory_word(2, 1.0)
            .with_memory_word(3, "ABCD")
            .with_instruction(
                4,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(4)
                    .with_address(103)
                    .build(),
            )
            .with_memory_word(110, 0o00000001)
            .with_memory_word(111, 0o17600000)
            .with_memory_word(112, 0o01020304)
            .with_memory_word(113, 0o10400147)
            .with_program_counter(105);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_pppp() {
        // Spec: "not used at present"
        assert!(true)
    }

    #[test]
    fn test_pin() {
        let program = r#"
0100            PIN  IO
0101            PIN  IO
0102            PIN  IO
0110    IO:     +0
"#;
        let actual = execute_io(program, "12", "12DATA*").ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::PIN)
                    .with_address(110)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::PIN)
                    .with_address(110)
                    .build(),
            )
            .with_instruction(
                102,
                InstructionBuilder::new(Function::PIN)
                    .with_address(110)
                    .build(),
            )
            .with_memory_word(110, "2")
            .with_program_counter(103);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_jump() {
        let program = r#"
0100    JUMP    1, 110
0101    TAKE    2, +1
0102    JUMP    1, 120
0110    TAKE    2, +2
0111    JUMP    1, 121
0120    TAKE    2, +3
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                InstructionBuilder::new(Function::JUMP)
                    .with_accumulator(1)
                    .with_address(110)
                    .build(),
            )
            .with_instruction(
                101,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 1)
                    .build(),
            )
            .with_instruction(
                102,
                InstructionBuilder::new(Function::JUMP)
                    .with_accumulator(1)
                    .with_address(120)
                    .build(),
            )
            .with_instruction(
                110,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2)
                    .build(),
            )
            .with_instruction(
                111,
                InstructionBuilder::new(Function::JUMP)
                    .with_accumulator(1)
                    .with_address(121)
                    .build(),
            )
            .with_instruction(
                120,
                InstructionBuilder::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 3)
                    .build(),
            )
            .with_memory_word(0, 111)
            .with_memory_word(2, 2)
            .with_memory_word(MEMORY_SIZE - 1, 1)
            .with_memory_word(MEMORY_SIZE - 2, 2)
            .with_memory_word(MEMORY_SIZE - 3, 3)
            .with_program_counter(121);
        assert_eq!(actual, expected)
    }
}
