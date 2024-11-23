use super::assembly::Assembly;
use super::charset::CharSet;
use super::memory::*;

use crate::result::Result;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Executor {
    execution_context: ExecutionContext,
    #[allow(dead_code)] // TODO: Remove
    stdin: Rc<RefCell<dyn std::io::Read>>,
    stdout: Rc<RefCell<dyn std::io::Write>>,
}

impl Executor {
    pub fn new() -> Self {
        let stdin = Rc::new(RefCell::new(std::io::stdin()));
        let stdout = Rc::new(RefCell::new(std::io::stdout()));
        Self::with_io(stdin, stdout)
    }

    pub fn with_io<R, W>(stdin: Rc<RefCell<R>>, stdout: Rc<RefCell<W>>) -> Self
    where
        R: std::io::Read + 'static,
        W: std::io::Write + 'static,
    {
        Self {
            execution_context: ExecutionContext::default(),
            stdin,
            stdout,
        }
    }

    pub fn execute(mut self, assembly: &Assembly) -> Result<ExecutionContext> {
        self.execution_context = assembly.clone().into();
        while self.can_step() {
            self.step();
        }
        Ok(self.execution_context.clone())
    }

    fn can_step(&self) -> bool {
        let context = &self.execution_context;
        let content = &context.memory[context.program_counter];
        matches!(content, Word::PWord(_))
    }

    fn step(&mut self) {
        let program_counter = self.execution_context.program_counter;
        self.execution_context.program_counter += 1;
        let content = self.execution_context.memory[program_counter];
        match content {
            Word::PWord(instruction) => self.step_word(&instruction),
            _ => unreachable!(),
        }
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
            // (Function::PTYP, Executor::exec_ptyp as ExecFn),
        ]
        .into_iter()
        .collect();

        let f = execution
            .get(&instruction.function())
            .expect("Expected instruction to be implemented");
        f(self, instruction)
    }

    fn operand(&self, instruction: &Instruction) -> Word {
        let memory = &self.execution_context.memory;

        let indirect = instruction.indirect();
        let index_register = instruction.index_register();

        let mut address = instruction.address();

        if indirect {
            address = match memory[address] {
                Word::PWord(instruction) => instruction.address(),
                _ => panic!("Indirect address must be another PWord"),
            };
        }

        if index_register != 0 {
            assert!(index_register < 7);
            address += index_register
        }

        memory[address]
    }

    fn exec_nil(&mut self, _instruction: &Instruction) {}

    fn exec_or(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        self.execution_context.memory[acc] |= operand;
    }

    fn exec_neqv(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        self.execution_context.memory[acc] ^= operand;
    }

    fn exec_and(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        self.execution_context.memory[acc] &= operand;
    }

    fn exec_add(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        self.execution_context.memory[acc] += operand;
    }

    fn exec_subt(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        self.execution_context.memory[acc] -= operand;
    }

    fn exec_mult(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        self.execution_context.memory[acc] *= operand;
    }

    fn exec_dvd(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        self.execution_context.memory[acc] /= operand;
    }

    fn exec_take(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        self.execution_context.memory[acc] = operand;
    }

    fn exec_tstr(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        if let Word::IWord(i) = operand {
            self.execution_context.memory[acc] = operand;
            self.execution_context.memory[acc - 1] = (if i < 1 { -1 } else { 0 }).into();
        } else {
            panic!("Invalid operand {:?} for TSTR", operand);
        }
    }

    fn exec_tneg(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        self.execution_context.memory[acc] = -operand;
    }

    fn exec_tnot(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        self.execution_context.memory[acc] = !operand;
    }

    fn exec_ttyp(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        let result = match operand {
            Word::IWord(_) => 0.into(),
            Word::FWord(_) => 1.into(),
            Word::SWord(_) => 2.into(),
            Word::PWord(_) => 3.into(),
            _ => panic!("Invalid operand {:?} for TTYP", operand),
        };
        self.execution_context.memory[acc] = result;
    }

    fn exec_ttyz(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        self.execution_context.memory[acc] = (operand.raw_bits() as i64).into();
    }

    fn exec_tout(&mut self, instruction: &Instruction) {
        let operand = self.operand(instruction);
        let bits = operand.raw_bits() & 0o77;
        let char = vec![CharSet::bits_to_char(bits).unwrap()];
        let mut stdout = (*self.stdout).borrow_mut();
        stdout.write_all(&char).unwrap();
    }

    fn exec_skip(&mut self, _instruction: &Instruction) {
        self.execution_context.program_counter += 1;
    }

    fn exec_skae(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        if self.execution_context.memory[acc] == operand {
            self.execution_context.program_counter += 1;
        }
    }

    fn exec_skan(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        if self.execution_context.memory[acc] != operand {
            self.execution_context.program_counter += 1;
        }
    }

    fn exec_sket(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        let same_type = match &self.execution_context.memory[acc] {
            Word::Undefined => matches!(operand, Word::Undefined),
            Word::IWord(_) => matches!(operand, Word::IWord(_)),
            Word::FWord(_) => matches!(operand, Word::FWord(_)),
            Word::SWord(_) => matches!(operand, Word::SWord(_)),
            Word::PWord(_) => matches!(operand, Word::PWord(_)),
        };
        if same_type {
            self.execution_context.program_counter += 1;
        }
    }

    fn exec_skal(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        if self.execution_context.memory[acc] < operand {
            self.execution_context.program_counter += 1
        }
    }

    fn exec_skag(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        if self.execution_context.memory[acc] > operand {
            self.execution_context.program_counter += 1
        }
    }

    fn exec_sked(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        if self.execution_context.memory[acc] == operand {
            self.execution_context.program_counter += 1
        } else {
            self.execution_context.memory[acc] -= 1.into()
        }
    }

    fn exec_skei(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        if self.execution_context.memory[acc] == operand {
            self.execution_context.program_counter += 1
        } else {
            self.execution_context.memory[acc] += 1.into()
        }
    }

    fn exec_shl(&mut self, instruction: &Instruction) {
        // TODO: Right shift
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        match operand {
            Word::IWord(_) => self.execution_context.memory[acc] <<= operand,
            _ => panic!("SHL requires IWord operand"),
        }
    }

    fn exec_rot(&mut self, instruction: &Instruction) {
        // TODO: Right rotate
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        match operand {
            Word::IWord(n) => self.execution_context.memory[acc].rotate(n),
            _ => panic!("ROT requires IWord operand"),
        }
    }

    fn exec_dshl(&mut self, instruction: &Instruction) {
        // TODO: Right shift
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);

        match operand {
            Word::IWord(n) => {
                let content = self.execution_context.memory[acc].raw_bits();
                let lsw = (content << n) & WORD_MASK;
                let msw = ((content << n) & OVERFLOW_MASK) >> 24;
                self.execution_context.memory[acc] =
                    self.execution_context.memory[acc].same_type_from(lsw);
                self.execution_context.memory[acc - 1] =
                    self.execution_context.memory[acc - 1].same_type_from(msw);
            }
            _ => panic!("DSHL requires IWord operand"),
        }
    }

    fn exec_drot(&mut self, instruction: &Instruction) {
        // TODO: Right rotate
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);

        match operand {
            Word::IWord(n) => {
                let msw = self.execution_context.memory[acc - 1].raw_bits();
                let lsw = self.execution_context.memory[acc].raw_bits();

                let shifted_msw = msw << n;
                let overflowed_msw = (shifted_msw & OVERFLOW_MASK) >> WORD_SIZE;

                let shifted_lsw = lsw << n;
                let overflowed_lsw = (shifted_lsw & OVERFLOW_MASK) >> WORD_SIZE;

                let updated_msw = (shifted_msw & WORD_MASK) | overflowed_lsw;
                let updated_lsw = (shifted_lsw & WORD_MASK) | overflowed_msw;

                self.execution_context.memory[acc] =
                    self.execution_context.memory[acc].same_type_from(updated_lsw);
                self.execution_context.memory[acc - 1] =
                    self.execution_context.memory[acc - 1].same_type_from(updated_msw);
            }
            _ => panic!("DROT requires IWord operand"),
        }
    }

    fn exec_powr(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        self.execution_context.memory[acc].power(operand);
    }

    fn exec_dmult(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let lhs_msw = self.execution_context.memory[acc - 1];
        let lhs_lsw = self.execution_context.memory[acc];
        let rhs = self.operand(instruction);
        match (lhs_msw, lhs_lsw, rhs) {
            (Word::IWord(_), Word::IWord(_), Word::IWord(rhs)) => {
                let lhs = Self::msw_lsw_to_i64(lhs_msw, lhs_lsw);
                let result = lhs * rhs;
                let (msw, lsw) = Self::i64_to_msw_lsw(result);
                self.execution_context.memory[acc - 1] = msw;
                self.execution_context.memory[acc] = lsw;
            }
            _ => panic!("DMULT requires IWord operands"),
        }
    }

    fn exec_div(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        self.execution_context.memory[acc] /= operand;
    }

    fn msw_lsw_to_i64(msw: Word, lsw: Word) -> i64 {
        println!(
            "m: {:?} mr: {:#010o} l: {:?} lr: {:#010o}",
            msw,
            msw.raw_bits(),
            lsw,
            lsw.raw_bits()
        );
        ((msw.raw_bits() << 40) as i64 >> 40) * 2_i64.pow(WORD_SIZE as u32)
            | (lsw.raw_bits() as i64)
    }

    fn i64_to_msw_lsw(value: i64) -> (Word, Word) {
        let lsw = ((value & WORD_MASK as i64) << 40) >> 40;
        let msw = (value & !WORD_MASK as i64) >> WORD_SIZE;
        (
            Word::IWord((msw << 40) >> 40),
            Word::IWord((lsw << 40) >> 40),
        )
    }

    fn exec_ddiv(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let lhs_msw = self.execution_context.memory[acc - 1];
        let lhs_lsw = self.execution_context.memory[acc];
        let rhs = self.operand(instruction);
        match (lhs_msw, lhs_lsw, rhs) {
            (Word::IWord(_), Word::IWord(_), Word::IWord(rhs)) => {
                let lhs = Self::msw_lsw_to_i64(lhs_msw, lhs_lsw);
                let result = lhs / rhs;
                let (msw, lsw) = Self::i64_to_msw_lsw(result);
                self.execution_context.memory[acc - 1] = msw;
                self.execution_context.memory[acc] = lsw;
            }
            _ => panic!("DMULT requires IWord operands"),
        }
    }

    fn exec_nilx(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let acc_value = self.execution_context.memory[acc];
        let operand = self.operand(instruction);
        let operand_address = instruction.address();
        self.execution_context.memory[acc] = operand;
        self.execution_context.memory[operand_address] = acc_value;
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
        let acc = instruction.accumulator();
        let acc_value = self.execution_context.memory[acc];
        let address = instruction.address();
        self.execution_context.memory[address] = acc_value;
    }

    fn exec_psqu(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let lhs_msw = self.execution_context.memory[acc - 1];
        let lhs_lsw = self.execution_context.memory[acc];
        match (lhs_msw, lhs_lsw) {
            (Word::IWord(l), Word::IWord(r)) => {
                assert!(if r >= 0 { l == 0 } else { l == -1 });
                let squashed = Self::msw_lsw_to_i64(lhs_msw, lhs_lsw);
                let address = instruction.address();
                self.execution_context.memory[address] = squashed.into();
            }
            _ => panic!("DMULT requires IWord operands"),
        }
    }

    fn exec_pneg(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let acc_value = self.execution_context.memory[acc];
        let address = instruction.address();
        self.execution_context.memory[address] = -acc_value;
    }

    fn exec_ptyp(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let acc_value = self.execution_context.memory[acc];
        let address = instruction.address();
        let address_value = self.execution_context.memory[address].raw_bits();
        match acc_value {
            Word::IWord(_) => {
                self.execution_context.memory[address] = acc_value.same_type_from(address_value)
            }
            Word::FWord(_) => {
                self.execution_context.memory[address] = acc_value.same_type_from(address_value)
            }
            Word::SWord(_) => {
                self.execution_context.memory[address] = acc_value.same_type_from(address_value)
            }
            Word::PWord(_) => {
                self.execution_context.memory[address] = acc_value.same_type_from(address_value)
            }
            Word::Undefined => panic!("PTYP source is not defined"),
        }
    }
}

impl std::fmt::Debug for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Executor")
            .field("execution_context", &self.execution_context)
            .field("stdin", &"Box<std::io::Read>")
            .field("stdout", &"Box<std::io::Write>")
            .finish()
    }
}

impl PartialEq for Executor {
    fn eq(&self, other: &Self) -> bool {
        self.execution_context.eq(&other.execution_context)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExecutionContext {
    program_counter: Offset,
    memory: Memory,
}

#[cfg(test)]
impl ExecutionContext {
    fn with_program_counter(self, program_counter: Offset) -> Self {
        Self {
            program_counter,
            ..self
        }
    }

    fn with_memory_word(mut self, offset: Offset, value: Word) -> Self {
        self.memory[offset] = value;
        self
    }

    fn with_instruction(self, location: Offset, instruction: Instruction) -> Self {
        self.with_memory_word(location, Word::PWord(instruction))
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
        let execution_context = do_execute(input, executor).unwrap().clone();

        let stdout = stdout.borrow();
        let bytes = stdout.buffer();
        let actual = String::from_utf8_lossy(&bytes);

        assert_eq!(actual, String::from(expected));

        Ok(execution_context)
    }

    fn do_execute(input: &str, executor: Executor) -> Result<ExecutionContext> {
        let program = input
            .to_string()
            .lines()
            .map(Parser::parse_line)
            .filter_map(|l| l.ok())
            .collect::<Vec<_>>();
        let assembly = Assembler::assemble(&program).expect("Valid assembly required");
        let execution_context = executor
            .execute(&assembly)
            .expect("Valid execution required");
        Ok(execution_context.clone())
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
                Instruction::new(Function::NIL)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, 3.14.into())
            .with_memory_word(MEMORY_SIZE - 1, 2.71.into())
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
                Instruction::new(Function::OR)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, 14.into())
            .with_memory_word(MEMORY_SIZE - 1, 10.into())
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
                Instruction::new(Function::NEQV)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, 6.into())
            .with_memory_word(MEMORY_SIZE - 1, 10.into())
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
                Instruction::new(Function::AND)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, 8.into())
            .with_memory_word(MEMORY_SIZE - 1, 10.into())
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
                Instruction::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, 22.into())
            .with_memory_word(MEMORY_SIZE - 1, 10.into())
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
                Instruction::new(Function::SUBT)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::SUBT)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_memory_word(1, 2.into())
            .with_memory_word(2, (-2).into())
            .with_memory_word(MEMORY_SIZE - 1, 10.into())
            .with_memory_word(MEMORY_SIZE - 2, 12.into())
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
                Instruction::new(Function::MULT)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, 120.into())
            .with_memory_word(MEMORY_SIZE - 1, 10.into())
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
                Instruction::new(Function::DVD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, 2.into())
            .with_memory_word(MEMORY_SIZE - 1, 6.into())
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
                Instruction::new(Function::TAKE)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_instruction(
                102,
                Instruction::new(Function::TAKE)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 3),
            )
            .with_instruction(
                103,
                Instruction::new(Function::TAKE)
                    .with_accumulator(4)
                    .with_address(110),
            )
            .with_memory_word(1, 42.into())
            .with_memory_word(2, 3.14.into())
            .with_memory_word(3, "ABCD".into())
            .with_memory_word(4, 2.718.into())
            .with_memory_word(MEMORY_SIZE - 1, 42.into())
            .with_memory_word(MEMORY_SIZE - 2, 3.14.into())
            .with_memory_word(MEMORY_SIZE - 3, "ABCD".into())
            .with_memory_word(110, 2.718.into())
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
                Instruction::new(Function::TSTR)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::TSTR)
                    .with_accumulator(4)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_memory_word(1, 0.into())
            .with_memory_word(2, 2.into())
            .with_memory_word(3, (-1).into())
            .with_memory_word(4, (-2).into())
            .with_memory_word(MEMORY_SIZE - 1, 2.into())
            .with_memory_word(MEMORY_SIZE - 2, (-2).into())
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
                Instruction::new(Function::TNEG)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::TNEG)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_memory_word(1, (-6).into())
            .with_memory_word(2, 6.into())
            .with_memory_word(MEMORY_SIZE - 1, 6.into())
            .with_memory_word(MEMORY_SIZE - 2, (-6).into())
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
                Instruction::new(Function::TNOT)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::TNOT)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_memory_word(1, (-5592406).into())
            .with_memory_word(2, 5592405.into())
            .with_memory_word(MEMORY_SIZE - 1, 5592405.into())
            .with_memory_word(MEMORY_SIZE - 2, (-5592406).into())
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
                Instruction::new(Function::TTYP)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::TTYP)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_instruction(
                102,
                Instruction::new(Function::TTYP)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 3),
            )
            .with_instruction(
                103,
                Instruction::new(Function::TTYP)
                    .with_accumulator(4)
                    .with_address(103),
            )
            .with_memory_word(1, 0.into())
            .with_memory_word(2, 1.into())
            .with_memory_word(3, 2.into())
            .with_memory_word(4, 3.into())
            .with_memory_word(MEMORY_SIZE - 1, 42.into())
            .with_memory_word(MEMORY_SIZE - 2, 3.14.into())
            .with_memory_word(MEMORY_SIZE - 3, "ABCD".into())
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
                Instruction::new(Function::TTYZ)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::TTYZ)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_memory_word(1, 42.into())
            .with_memory_word(2, 0o01020304.into())
            .with_memory_word(MEMORY_SIZE - 1, 42.into())
            .with_memory_word(MEMORY_SIZE - 2, "ABCD".into())
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
                Instruction::new(Function::TOUT).with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::TOUT).with_address(MEMORY_SIZE - 2),
            )
            .with_memory_word(MEMORY_SIZE - 1, 1.into())
            .with_memory_word(MEMORY_SIZE - 2, "ABCD".into())
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
                Instruction::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(101, Instruction::new(Function::SKIP))
            .with_instruction(
                102,
                Instruction::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_memory_word(1, 1.into())
            .with_memory_word(MEMORY_SIZE - 1, 1.into())
            .with_memory_word(MEMORY_SIZE - 2, 1.into())
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
                Instruction::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::SKAE)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_instruction(
                102,
                Instruction::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 3),
            )
            .with_instruction(
                103,
                Instruction::new(Function::ADD)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 4),
            )
            .with_instruction(
                104,
                Instruction::new(Function::SKAE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 5),
            )
            .with_instruction(
                105,
                Instruction::new(Function::ADD)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 6),
            )
            .with_memory_word(1, 1.into())
            .with_memory_word(2, 2.into())
            .with_memory_word(MEMORY_SIZE - 1, 1.into())
            .with_memory_word(MEMORY_SIZE - 2, 1.into())
            .with_memory_word(MEMORY_SIZE - 3, 1.into())
            .with_memory_word(MEMORY_SIZE - 4, 1.into())
            .with_memory_word(MEMORY_SIZE - 5, 2.into())
            .with_memory_word(MEMORY_SIZE - 6, 1.into())
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
                Instruction::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::SKAN)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_instruction(
                102,
                Instruction::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 3),
            )
            .with_instruction(
                103,
                Instruction::new(Function::ADD)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 4),
            )
            .with_instruction(
                104,
                Instruction::new(Function::SKAN)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 5),
            )
            .with_instruction(
                105,
                Instruction::new(Function::ADD)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 6),
            )
            .with_memory_word(1, 2.into())
            .with_memory_word(2, 1.into())
            .with_memory_word(MEMORY_SIZE - 1, 1.into())
            .with_memory_word(MEMORY_SIZE - 2, 1.into())
            .with_memory_word(MEMORY_SIZE - 3, 1.into())
            .with_memory_word(MEMORY_SIZE - 4, 1.into())
            .with_memory_word(MEMORY_SIZE - 5, 2.into())
            .with_memory_word(MEMORY_SIZE - 6, 1.into())
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
                Instruction::new(Function::TAKE)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::SKET)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_instruction(
                102,
                Instruction::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 3),
            )
            .with_instruction(
                103,
                Instruction::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 4),
            )
            .with_instruction(
                104,
                Instruction::new(Function::SKET)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 5),
            )
            .with_instruction(
                105,
                Instruction::new(Function::ADD)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 6),
            )
            .with_memory_word(1, 1.0.into())
            .with_memory_word(2, 2.0.into())
            .with_memory_word(MEMORY_SIZE - 1, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 2, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 3, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 4, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 5, 1.into())
            .with_memory_word(MEMORY_SIZE - 6, 1.0.into())
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
                Instruction::new(Function::TAKE)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::SKAL)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_instruction(
                102,
                Instruction::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 3),
            )
            .with_instruction(
                103,
                Instruction::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 4),
            )
            .with_instruction(
                104,
                Instruction::new(Function::SKAL)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 5),
            )
            .with_instruction(
                105,
                Instruction::new(Function::ADD)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 6),
            )
            .with_instruction(
                106,
                Instruction::new(Function::TAKE)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 7),
            )
            .with_instruction(
                107,
                Instruction::new(Function::SKAL)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 8),
            )
            .with_instruction(
                108,
                Instruction::new(Function::ADD)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 9),
            )
            .with_memory_word(1, 0.0.into())
            .with_memory_word(2, 2.0.into())
            .with_memory_word(3, 3.0.into())
            .with_memory_word(MEMORY_SIZE - 1, 0.0.into())
            .with_memory_word(MEMORY_SIZE - 2, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 3, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 4, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 5, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 6, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 7, 2.0.into())
            .with_memory_word(MEMORY_SIZE - 8, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 9, 1.0.into())
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
                Instruction::new(Function::TAKE)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::SKAG)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_instruction(
                102,
                Instruction::new(Function::ADD)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 3),
            )
            .with_instruction(
                103,
                Instruction::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 4),
            )
            .with_instruction(
                104,
                Instruction::new(Function::SKAG)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 5),
            )
            .with_instruction(
                105,
                Instruction::new(Function::ADD)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 6),
            )
            .with_instruction(
                106,
                Instruction::new(Function::TAKE)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 7),
            )
            .with_instruction(
                107,
                Instruction::new(Function::SKAG)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 8),
            )
            .with_instruction(
                108,
                Instruction::new(Function::ADD)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 9),
            )
            .with_memory_word(1, 1.0.into())
            .with_memory_word(2, 2.0.into())
            .with_memory_word(3, 2.0.into())
            .with_memory_word(MEMORY_SIZE - 1, 0.0.into())
            .with_memory_word(MEMORY_SIZE - 2, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 3, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 4, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 5, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 6, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 7, 2.0.into())
            .with_memory_word(MEMORY_SIZE - 8, 1.0.into())
            .with_memory_word(MEMORY_SIZE - 9, 1.0.into())
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
                Instruction::new(Function::TAKE)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::SKED)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_instruction(102, Instruction::new(Function::NIL))
            .with_instruction(
                103,
                Instruction::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 3),
            )
            .with_instruction(
                104,
                Instruction::new(Function::SKED)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 4),
            )
            .with_memory_word(1, 42.into())
            .with_memory_word(2, 0.into())
            .with_memory_word(MEMORY_SIZE - 1, 42.into())
            .with_memory_word(MEMORY_SIZE - 2, 42.into())
            .with_memory_word(MEMORY_SIZE - 3, 1.into())
            .with_memory_word(MEMORY_SIZE - 4, 42.into())
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
                Instruction::new(Function::TAKE)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::SKEI)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_instruction(102, Instruction::new(Function::NIL))
            .with_instruction(
                103,
                Instruction::new(Function::TAKE)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 3),
            )
            .with_instruction(
                104,
                Instruction::new(Function::SKEI)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 4),
            )
            .with_memory_word(1, 42.into())
            .with_memory_word(2, 2.into())
            .with_memory_word(MEMORY_SIZE - 1, 42.into())
            .with_memory_word(MEMORY_SIZE - 2, 42.into())
            .with_memory_word(MEMORY_SIZE - 3, 1.into())
            .with_memory_word(MEMORY_SIZE - 4, 42.into())
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
                Instruction::new(Function::SHL)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::SHL)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_instruction(
                102,
                Instruction::new(Function::SHL)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 3),
            )
            .with_memory_word(1, 2.into())
            .with_memory_word(2, 9.223372036854776e18.into())
            .with_memory_word(3, " AB\0".into())
            .with_memory_word(MEMORY_SIZE - 1, 1.into())
            .with_memory_word(MEMORY_SIZE - 2, 1.into())
            .with_memory_word(MEMORY_SIZE - 3, 6.into())
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
                Instruction::new(Function::ROT)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::ROT)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_instruction(
                102,
                Instruction::new(Function::ROT)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 3),
            )
            .with_memory_word(1, 2.into())
            .with_memory_word(2, 9.223372036854776e18.into())
            .with_memory_word(3, "BCDA".into())
            .with_memory_word(MEMORY_SIZE - 1, 25.into())
            .with_memory_word(MEMORY_SIZE - 2, 1.into())
            .with_memory_word(MEMORY_SIZE - 3, 6.into())
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
                Instruction::new(Function::DSHL)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::DSHL)
                    .with_accumulator(4)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_memory_word(1, 2.into())
            .with_memory_word(2, 0.into())
            .with_memory_word(3, "AB".into())
            .with_memory_word(4, "CD\0\0".into())
            .with_memory_word(MEMORY_SIZE - 1, 25.into())
            .with_memory_word(MEMORY_SIZE - 2, 12.into())
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
                Instruction::new(Function::DROT)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::DROT)
                    .with_accumulator(4)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_memory_word(1, 2.into())
            .with_memory_word(2, 4.into())
            .with_memory_word(3, "YZAB".into())
            .with_memory_word(4, "CDWX".into())
            .with_memory_word(MEMORY_SIZE - 1, 25.into())
            .with_memory_word(MEMORY_SIZE - 2, 12.into())
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
                Instruction::new(Function::POWR)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::POWR)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_memory_word(1, 8.into())
            .with_memory_word(2, 8.0.into())
            .with_memory_word(MEMORY_SIZE - 1, 3.into())
            .with_memory_word(MEMORY_SIZE - 2, 3.0.into())
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
                Instruction::new(Function::DMULT)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, (-12).into())
            .with_memory_word(2, (-7450624).into())
            .with_memory_word(MEMORY_SIZE - 1, 12000.into())
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
                Instruction::new(Function::DIV)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::DIV)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_memory_word(1, (-6).into())
            .with_memory_word(2, 5.into())
            .with_memory_word(MEMORY_SIZE - 1, 7.into())
            .with_memory_word(MEMORY_SIZE - 2, 7.into())
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
                Instruction::new(Function::DDIV)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, 0.into())
            .with_memory_word(2, 16000.into())
            .with_memory_word(MEMORY_SIZE - 1, (-12000).into())
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
                Instruction::new(Function::NILX)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, 2.71.into())
            .with_memory_word(MEMORY_SIZE - 1, 3.14.into())
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
                Instruction::new(Function::ORX)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, 10.into())
            .with_memory_word(MEMORY_SIZE - 1, 14.into())
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
                Instruction::new(Function::NEQVX)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, 10.into())
            .with_memory_word(MEMORY_SIZE - 1, 6.into())
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
                Instruction::new(Function::ADDX)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, 10.into())
            .with_memory_word(MEMORY_SIZE - 1, 22.into())
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
                Instruction::new(Function::SUBTX)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::SUBTX)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_memory_word(1, 10.into())
            .with_memory_word(2, 12.into())
            .with_memory_word(MEMORY_SIZE - 1, 2.into())
            .with_memory_word(MEMORY_SIZE - 2, (-2).into())
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
                Instruction::new(Function::MULTX)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, 10.into())
            .with_memory_word(MEMORY_SIZE - 1, 120.into())
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
                Instruction::new(Function::DVDX)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_memory_word(1, 6.into())
            .with_memory_word(MEMORY_SIZE - 1, 2.into())
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
                Instruction::new(Function::PUT)
                    .with_accumulator(1)
                    .with_address(MEMORY_SIZE - 1),
            )
            .with_instruction(
                101,
                Instruction::new(Function::PUT)
                    .with_accumulator(2)
                    .with_address(MEMORY_SIZE - 2),
            )
            .with_instruction(
                102,
                Instruction::new(Function::PUT)
                    .with_accumulator(3)
                    .with_address(MEMORY_SIZE - 3),
            )
            .with_instruction(
                103,
                Instruction::new(Function::PUT)
                    .with_accumulator(4)
                    .with_address(110),
            )
            .with_memory_word(1, 42.into())
            .with_memory_word(2, 3.14.into())
            .with_memory_word(3, "ABCD".into())
            .with_memory_word(4, 2.718.into())
            .with_memory_word(MEMORY_SIZE - 1, 42.into())
            .with_memory_word(MEMORY_SIZE - 2, 3.14.into())
            .with_memory_word(MEMORY_SIZE - 3, "ABCD".into())
            .with_memory_word(110, 2.718.into())
            .with_program_counter(104);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_psqu() {
        let program = r#"
0001            -1
0002            -12
0100            PSQU 2, LOC
0110    LOC:     +0
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                Instruction::new(Function::PSQU)
                    .with_accumulator(2)
                    .with_address(110),
            )
            .with_memory_word(1, (-1).into())
            .with_memory_word(2, (-12).into())
            .with_memory_word(110, (-12).into())
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
0110    LOC1:    +0
0111    LOC2:    +0
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                Instruction::new(Function::PNEG)
                    .with_accumulator(1)
                    .with_address(110),
            )
            .with_instruction(
                101,
                Instruction::new(Function::PNEG)
                    .with_accumulator(2)
                    .with_address(111),
            )
            .with_memory_word(1, 3.into())
            .with_memory_word(2, (-3.14).into())
            .with_memory_word(110, (-3).into())
            .with_memory_word(111, 3.14.into())
            .with_program_counter(102);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_ptyp() {
        let program = r#"
0001    +0
0002    +1
0003    +2
0004    +3
0100            PTYP 1, LOC1
0101            PTYP 2, LOC2
0102            PTYP 3, LOC3
0103            PTYP 4, LOC4
0110    LOC1:   +0.0
0111    LOC2:   +0.0
0112    LOC3:   +0.0
0113    LOC4:   +0.0
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_instruction(
                100,
                Instruction::new(Function::PTYP)
                    .with_accumulator(1)
                    .with_address(110),
            )
            .with_instruction(
                101,
                Instruction::new(Function::PTYP)
                    .with_accumulator(2)
                    .with_address(111),
            )
            .with_instruction(
                102,
                Instruction::new(Function::PTYP)
                    .with_accumulator(3)
                    .with_address(112),
            )
            .with_instruction(
                103,
                Instruction::new(Function::PTYP)
                    .with_accumulator(4)
                    .with_address(113),
            )
            .with_memory_word(1, 42.into())
            .with_memory_word(2, 3.14.into())
            .with_memory_word(3, "ABCD".into())
            // .with_memory_word(4, 2.718.into())
            .with_memory_word(110, 0.into())
            .with_memory_word(111, 0.0.into())
            .with_memory_word(112, "\0\0\0\0".into())
            .with_memory_word(113, 0.into())
            .with_program_counter(104);
        assert_eq!(actual, expected)
    }
}
