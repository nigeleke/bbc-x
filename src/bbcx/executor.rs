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
            self.execution_context.memory[acc - 1] = Word::IWord(if i < 1 { -1 } else { 0 });
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
            Word::IWord(_) => Word::IWord(0),
            Word::FWord(_) => Word::IWord(1),
            Word::SWord(_) => Word::IWord(2),
            Word::PWord(_) => Word::IWord(3),
            _ => panic!("Invalid operand {:?} for TTYP", operand),
        };
        self.execution_context.memory[acc] = result;
    }

    fn exec_ttyz(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        self.execution_context.memory[acc] = Word::IWord(operand.raw_bits() as IntType);
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
        println!(
            "Checking acc: {} -> {:?} and operand: {:?} => {}",
            acc, self.execution_context.memory[acc], operand, same_type
        );
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
            self.execution_context.memory[acc] -= Word::IWord(1)
        }
    }

    fn exec_skei(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        if self.execution_context.memory[acc] == operand {
            self.execution_context.program_counter += 1
        } else {
            self.execution_context.memory[acc] += Word::IWord(1)
        }
    }

    fn exec_shl(&mut self, instruction: &Instruction) {
        let acc = instruction.accumulator();
        let operand = self.operand(instruction);
        println!(
            "exec_shl: {:?} => {:?} {:?}",
            acc, self.execution_context.memory[acc], operand
        );
        match operand {
            Word::IWord(_) => self.execution_context.memory[acc] <<= operand,
            _ => panic!("SHL requires IWord operand"),
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
        let assembly = Assembler::assemble(&program)?;
        let execution_context = executor.execute(&assembly)?;
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
0001    NIL
"#;
        let actual = execute(program).ok().unwrap();
        let expected = ExecutionContext::default()
            .with_program_counter(2)
            .with_instruction(1, Instruction::new(Function::NIL));
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
            .with_program_counter(101)
            .with_memory_word(1, Word::IWord(14))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(10));
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
            .with_program_counter(101)
            .with_memory_word(1, Word::IWord(6))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(10));
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
            .with_program_counter(101)
            .with_memory_word(1, Word::IWord(8))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(10));
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
            .with_program_counter(101)
            .with_memory_word(1, Word::IWord(22))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(10));
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
            .with_program_counter(102)
            .with_memory_word(1, Word::IWord(2))
            .with_memory_word(2, Word::IWord(-2))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(10))
            .with_memory_word(MEMORY_SIZE - 2, Word::IWord(12));
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
            .with_program_counter(101)
            .with_memory_word(1, Word::IWord(120))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(10));
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
            .with_program_counter(101)
            .with_memory_word(1, Word::IWord(2))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(6));
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
            .with_program_counter(104)
            .with_memory_word(1, Word::IWord(42))
            .with_memory_word(2, Word::FWord(3.14))
            .with_memory_word(3, "ABCD".into())
            .with_memory_word(4, Word::FWord(2.718))
            .with_memory_word(110, Word::FWord(2.718))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(42))
            .with_memory_word(MEMORY_SIZE - 2, Word::FWord(3.14))
            .with_memory_word(MEMORY_SIZE - 3, "ABCD".into());
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
            .with_program_counter(102)
            .with_memory_word(1, Word::IWord(0))
            .with_memory_word(2, Word::IWord(2))
            .with_memory_word(3, Word::IWord(-1))
            .with_memory_word(4, Word::IWord(-2))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(2))
            .with_memory_word(MEMORY_SIZE - 2, Word::IWord(-2));
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
            .with_program_counter(102)
            .with_memory_word(1, Word::IWord(-6))
            .with_memory_word(2, Word::IWord(6))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(6))
            .with_memory_word(MEMORY_SIZE - 2, Word::IWord(-6));
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
            .with_program_counter(102)
            .with_memory_word(1, Word::IWord(-5592406))
            .with_memory_word(2, Word::IWord(5592405))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(5592405))
            .with_memory_word(MEMORY_SIZE - 2, Word::IWord(-5592406));
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
            .with_program_counter(104)
            .with_memory_word(1, Word::IWord(0))
            .with_memory_word(2, Word::IWord(1))
            .with_memory_word(3, Word::IWord(2))
            .with_memory_word(4, Word::IWord(3))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(42))
            .with_memory_word(MEMORY_SIZE - 2, Word::FWord(3.14))
            .with_memory_word(MEMORY_SIZE - 3, "ABCD".into());
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
            .with_program_counter(102)
            .with_memory_word(1, Word::IWord(42))
            .with_memory_word(2, Word::IWord(0o01020304))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(42))
            .with_memory_word(MEMORY_SIZE - 2, "ABCD".into());
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
            .with_program_counter(102)
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 2, "ABCD".into());
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
            .with_program_counter(103)
            .with_memory_word(1, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 2, Word::IWord(1));
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
            .with_memory_word(1, Word::IWord(1))
            .with_memory_word(2, Word::IWord(2))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 2, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 3, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 4, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 5, Word::IWord(2))
            .with_memory_word(MEMORY_SIZE - 6, Word::IWord(1))
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
            .with_memory_word(1, Word::IWord(2))
            .with_memory_word(2, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 2, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 3, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 4, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 5, Word::IWord(2))
            .with_memory_word(MEMORY_SIZE - 6, Word::IWord(1))
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
            .with_memory_word(1, Word::FWord(1.0))
            .with_memory_word(2, Word::FWord(2.0))
            .with_memory_word(MEMORY_SIZE - 1, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 2, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 3, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 4, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 5, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 6, Word::FWord(1.0))
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
            .with_memory_word(1, Word::FWord(0.0))
            .with_memory_word(2, Word::FWord(2.0))
            .with_memory_word(3, Word::FWord(3.0))
            .with_memory_word(MEMORY_SIZE - 1, Word::FWord(0.0))
            .with_memory_word(MEMORY_SIZE - 2, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 3, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 4, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 5, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 6, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 7, Word::FWord(2.0))
            .with_memory_word(MEMORY_SIZE - 8, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 9, Word::FWord(1.0))
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
            .with_memory_word(1, Word::FWord(1.0))
            .with_memory_word(2, Word::FWord(2.0))
            .with_memory_word(3, Word::FWord(2.0))
            .with_memory_word(MEMORY_SIZE - 1, Word::FWord(0.0))
            .with_memory_word(MEMORY_SIZE - 2, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 3, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 4, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 5, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 6, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 7, Word::FWord(2.0))
            .with_memory_word(MEMORY_SIZE - 8, Word::FWord(1.0))
            .with_memory_word(MEMORY_SIZE - 9, Word::FWord(1.0))
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
            .with_memory_word(1, Word::IWord(42))
            .with_memory_word(2, Word::IWord(0))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(42))
            .with_memory_word(MEMORY_SIZE - 2, Word::IWord(42))
            .with_memory_word(MEMORY_SIZE - 3, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 4, Word::IWord(42))
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
            .with_memory_word(1, Word::IWord(42))
            .with_memory_word(2, Word::IWord(2))
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(42))
            .with_memory_word(MEMORY_SIZE - 2, Word::IWord(42))
            .with_memory_word(MEMORY_SIZE - 3, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 4, Word::IWord(42))
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
            .with_memory_word(1, Word::IWord(2))
            .with_memory_word(2, Word::FWord(9.223372036854776e18))
            .with_memory_word(3, " AB\0".into())
            .with_memory_word(MEMORY_SIZE - 1, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 2, Word::IWord(1))
            .with_memory_word(MEMORY_SIZE - 3, Word::IWord(6))
            .with_program_counter(103);
        assert_eq!(actual, expected)
    }
}
