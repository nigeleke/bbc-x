use std::fmt::{format, write};

#[derive(Clone, Debug, PartialEq)]
pub struct SourceProgramLine {
    location: Location,
    label: Label,
    source_program_word: SourceProgramWord,
    comment: Comment,
}

impl SourceProgramLine {
    pub fn new(
        location: Location,
        label: Label,
        source_program_word: SourceProgramWord,
        comment: Comment,
    ) -> Self {
        Self {
            location,
            label,
            source_program_word,
            comment,
        }
    }

    pub fn location(&self) -> &Location {
        &self.location
    }

    pub fn source_program_word(&self) -> &SourceProgramWord {
        &self.source_program_word
    }
}

impl std::fmt::Display for SourceProgramLine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        let location = format!("{:04}", self.location);
        let label = format!("{:08}", self.label);
        let source_program_word = self.source_program_word.to_string();
        let comment = self.comment.to_string();

        write!(f, "{:<8}{:<42}{}", location, source_program_word, comment)
    }
}

pub type Location = AddressRef;

#[derive(Clone, Debug, PartialEq)]
pub struct Label(Option<String>);

impl From<String> for Label {
    fn from(a: String) -> Self {
        Self(Some(a))
    }
}

impl From<Option<String>> for Label {
    fn from(a: Option<String>) -> Self {
        Self(a)
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(l) => write!(f, "{}:", l),
            None => write!(f, ""),
        }
    }
}

pub type Identifier = String;

#[derive(Clone, Debug, PartialEq)]
pub enum SourceProgramWord {
    PWord(PWord),
    FWord(FWord),
    IWord(IWord),
    SWord(SWord),
}

impl std::fmt::Display for SourceProgramWord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            SourceProgramWord::SWord(sword) => write!(f, "\"{}\"", sword),
            SourceProgramWord::PWord(pword) => write!(f, "{}", pword),
            SourceProgramWord::FWord(fword) => write!(f, "{}", fword),
            SourceProgramWord::IWord(iword) => write!(f, "{}", iword),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PWord {
    mnemonic: Mnemonic,
    accumulator: Acc,
    store_operand: StoreOperand,
}

impl PWord {
    pub fn new(mnemonic: Mnemonic, accumulator: Acc, store_operand: StoreOperand) -> Self {
        Self {
            mnemonic,
            accumulator,
            store_operand,
        }
    }
}

impl std::fmt::Display for PWord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{:<8}{:>2} {}",
            self.mnemonic.to_string(),
            self.accumulator.to_string(),
            self.store_operand.to_string()
        )
    }
}

pub type SWord = String;

// TODO: Line 86 in bbc-3
#[derive(Clone, Debug, PartialEq)]
pub struct Acc(Option<char>);

impl From<char> for Acc {
    fn from(a: char) -> Self {
        Self(Some(a))
    }
}

impl From<Option<char>> for Acc {
    fn from(a: Option<char>) -> Self {
        Self(a)
    }
}

impl std::fmt::Display for Acc {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.0.map_or("".into(), |a| a.to_string()))
    }
}

pub type FWord = FloatType;
pub type IWord = IntType;

// TODO:
#[derive(Clone, Debug, PartialEq)]
pub enum StoreOperand {
    None,
    AddressOperand(AddressOperand),
    ConstOperand(ConstOperand),
}

impl std::fmt::Display for StoreOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreOperand::None => write!(f, ""),
            StoreOperand::AddressOperand(o) => write!(f, "{}", o.to_string()),
            StoreOperand::ConstOperand(o) => write!(f, "{}", o.to_string()),
        }
    }
}

// TOOD: Line 110 in bbc-3
pub type Comment = String;

// TODO: Line 146 in bbc-3
#[derive(Clone, Debug, PartialEq)]
pub struct AddressOperand {
    address: SimpleAddressOperand,
    index: Option<Index>,
}

impl AddressOperand {
    pub fn new(address: SimpleAddressOperand, index: Option<Index>) -> Self {
        Self { address, index }
    }
}

impl std::fmt::Display for AddressOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        let index = self.index.map(|i| format!(":{}", i)).unwrap_or("".into());
        write!(f, "{}{}", self.address, index)
    }
}

// TODO: Line 186 in bbc-3
#[derive(Clone, Debug, PartialEq)]
pub enum ConstOperand {
    SignedInteger(IntType),
    SignedFWord(FloatType),
    SWord(SWord),
}

impl std::fmt::Display for ConstOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            ConstOperand::SignedInteger(c) => write!(f, "{:+}", c),
            ConstOperand::SignedFWord(c) => write!(f, "{:+}", c),
            ConstOperand::SWord(c) => write!(f, "<{}>", c),
        }
    }
}

// TODO: Line 186 in bbc-3
#[derive(Clone, Debug, PartialEq)]
pub enum SimpleAddressOperand {
    DirectAddress(Address),
    IndirectAddress(Address),
}

impl std::fmt::Display for SimpleAddressOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            SimpleAddressOperand::DirectAddress(a) => write!(f, "{}", a),
            SimpleAddressOperand::IndirectAddress(a) => write!(f, "*{}", a),
        }
    }
}

// TODO: Line 201 in bbc-3
#[derive(Clone, Debug, PartialEq)]
pub enum Address {
    Identifier(Identifier),
    NumericAddress(NumericAddress),
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Address::Identifier(i) => write!(f, "{}", i),
            Address::NumericAddress(a) => write!(f, "{}", a),
        }
    }
}

// TODO: Line 216 in bbc-3
pub type NumericAddress = u16;

// TODO: Line 230++ in bbc-3
pub type Character = char;
pub type NumericCharacter = char;
pub type Punctuation = char;
pub type IntType = i32;
pub type FloatType = f32;
pub type AddressRef = usize;
pub type Index = usize;

// TODO: Line 244 in bbc-3
#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Mnemonic {
    NTHG,
    OR,
    NEQV,
    AND,
    ADDX,
    ADD,
    SUBT,
    MPLY,
    DVD,
    TAKE,
    TSTR,
    TNEG,
    TNOT,
    TTYP,
    TTYZ,
    TTTT,
    TOUT,
    SKIP,
    SKAE,
    SKAN,
    SKET,
    SKAL,
    SKAG,
    SKED,
    SKEI,
    SHL,
    ROT,
    DSHL,
    DROT,
    POWR,
    DMULT,
    DIV,
    DDIV,
    SWAP,
    ORX,
    NEQVX,
    ANDX,
    // ADDX,
    SUBTX,
    MPLYX,
    DVDX,
    PUT,
    PSQU,
    PNEG,
    PNOT,
    PTYP,
    PTYZ,
    PFFP,
    PIN,
    JUMP,
    JEZ,
    JNZ,
    JAT,
    JLZ,
    JGZ,
    JZD,
    JZI,
    DECR,
    INCR,
    MOCKP,
    MOCKS,
    DBYTE,
    EXEC,
    EXTRA,
    // Library
    SQRT,
    LN,
    EXP,
    READ,
    PRINT,
    SIN,
    COS,
    TAN,
    ATN,
    STOP,
    LINE,
    INT,
    FRAC,
    FLOAT,
    CAPTN,
    PAGE,
    RND,
    ABS,
}

impl std::fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
