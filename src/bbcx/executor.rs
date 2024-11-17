use super::assembly::Assembly;
use super::ast::*;
use super::charset::CharSet;
use super::memory::*;

use crate::result::Result;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Executor {
    execution_context: ExecutionContext,
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
            (Mnemonic::TSTR, Executor::exec_tstr as ExecFn),
            (Mnemonic::TNEG, Executor::exec_tneg as ExecFn),
            (Mnemonic::TNOT, Executor::exec_tnot as ExecFn),
            (Mnemonic::TTYP, Executor::exec_ttyp as ExecFn),
            (Mnemonic::TTYZ, Executor::exec_ttyz as ExecFn),
            (Mnemonic::TOUT, Executor::exec_tout as ExecFn),
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
                ConstOperand::SignedIWord(i) => MemoryWord::Integer(*i),
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

    fn exec_tstr(&mut self, acc: Location, operand: MemoryWord) {
        if let MemoryWord::Integer(i) = operand {
            self.execution_context.memory[acc] = operand;
            self.execution_context.memory[acc - 1] =
                MemoryWord::Integer(if i < 1 { -1 } else { 0 });
        } else {
            panic!("Invalid operand {:?} for TSTR", operand);
        }
    }

    fn exec_tneg(&mut self, acc: Location, operand: MemoryWord) {
        self.execution_context.memory[acc] = -operand;
    }

    fn exec_tnot(&mut self, acc: Location, operand: MemoryWord) {
        self.execution_context.memory[acc] = !operand;
    }

    fn exec_ttyp(&mut self, acc: Location, operand: MemoryWord) {
        let result = match operand {
            MemoryWord::Integer(_) => MemoryWord::Integer(0),
            MemoryWord::Float(_) => MemoryWord::Integer(1),
            MemoryWord::String(_) => MemoryWord::Integer(2),
            MemoryWord::Instruction(_) => MemoryWord::Integer(3),
            _ => panic!("Invalid operand {:?} for TTYP", operand),
        };
        self.execution_context.memory[acc] = result;
    }

    fn exec_ttyz(&mut self, acc: Location, operand: MemoryWord) {
        self.execution_context.memory[acc] = MemoryWord::Integer(operand.to_24_bits() as i64);
    }

    fn exec_tout(&mut self, _acc: Location, operand: MemoryWord) {
        let bits = operand.to_24_bits() & 0o77;
        let char = vec![CharSet::bits_to_char(bits).unwrap()];
        let mut stdout = (*self.stdout).borrow_mut();
        stdout.write(&char).unwrap();
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

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutionContext {
    program_counter: Location,
    memory: Memory,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            program_counter: Location::default(),
            memory: Memory::default(),
        }
    }
}

#[cfg(test)]
impl ExecutionContext {
    fn with_program_counter(self, location: Location) -> Self {
        Self {
            program_counter: location,
            ..self
        }
    }

    fn with_memory_word(mut self, location: Location, value: MemoryWord) -> Self {
        self.memory[location] = value;
        self
    }

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
            ..Self::default()
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
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(10)),
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
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(10)),
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
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(10)),
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
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(10)),
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
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(10)),
                ),
            )
            .with_instruction(
                101,
                PWord::new(
                    Mnemonic::SUBT,
                    '2'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(12)),
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
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(10)),
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
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(6)),
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
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(42)),
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
                PWord::new(
                    Mnemonic::TSTR,
                    '2'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(2)),
                ),
            )
            .with_instruction(
                101,
                PWord::new(
                    Mnemonic::TSTR,
                    '4'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(-2)),
                ),
            )
            .with_program_counter(102)
            .with_memory_word(1, MemoryWord::Integer(0))
            .with_memory_word(2, MemoryWord::Integer(2))
            .with_memory_word(3, MemoryWord::Integer(-1))
            .with_memory_word(4, MemoryWord::Integer(-2));
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
                PWord::new(
                    Mnemonic::TNEG,
                    '1'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(6)),
                ),
            )
            .with_instruction(
                101,
                PWord::new(
                    Mnemonic::TNEG,
                    '2'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(-6)),
                ),
            )
            .with_program_counter(102)
            .with_memory_word(1, MemoryWord::Integer(-6))
            .with_memory_word(2, MemoryWord::Integer(6));
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
                PWord::new(
                    Mnemonic::TNOT,
                    '1'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(5592405)),
                ),
            )
            .with_instruction(
                101,
                PWord::new(
                    Mnemonic::TNOT,
                    '2'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(-5592406)),
                ),
            )
            .with_program_counter(102)
            .with_memory_word(1, MemoryWord::Integer(-5592406))
            .with_memory_word(2, MemoryWord::Integer(5592405));
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
                PWord::new(
                    Mnemonic::TTYP,
                    '1'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(42)),
                ),
            )
            .with_instruction(
                101,
                PWord::new(
                    Mnemonic::TTYP,
                    '2'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedFWord(3.14)),
                ),
            )
            .with_instruction(
                102,
                PWord::new(
                    Mnemonic::TTYP,
                    '3'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SWord("ABCD".into())),
                ),
            )
            .with_instruction(
                103,
                PWord::new(
                    Mnemonic::TTYP,
                    '4'.into(),
                    StoreOperand::AddressOperand(AddressOperand::new(
                        SimpleAddressOperand::DirectAddress(Address::NumericAddress(103)),
                        None,
                    )),
                ),
            )
            .with_program_counter(104)
            .with_memory_word(1, MemoryWord::Integer(0))
            .with_memory_word(2, MemoryWord::Integer(1))
            .with_memory_word(3, MemoryWord::Integer(2))
            .with_memory_word(4, MemoryWord::Integer(3));
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
                PWord::new(
                    Mnemonic::TTYZ,
                    '1'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(42)),
                ),
            )
            .with_instruction(
                101,
                PWord::new(
                    Mnemonic::TTYZ,
                    '2'.into(),
                    StoreOperand::ConstOperand(ConstOperand::SWord("ABCD".into())),
                ),
            )
            .with_program_counter(102)
            .with_memory_word(1, MemoryWord::Integer(42))
            .with_memory_word(2, MemoryWord::Integer(0o01020304));
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
                PWord::new(
                    Mnemonic::TOUT,
                    None.into(),
                    StoreOperand::ConstOperand(ConstOperand::SignedIWord(1)),
                ),
            )
            .with_instruction(
                101,
                PWord::new(
                    Mnemonic::TOUT,
                    None.into(),
                    StoreOperand::ConstOperand(ConstOperand::SWord("ABCD".into())),
                ),
            )
            .with_program_counter(102);
        assert_eq!(actual, expected)
    }
}
