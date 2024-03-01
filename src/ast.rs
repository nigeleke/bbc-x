pub(crate) type SourceProgram = Vec<SourceProgramLine>;

#[derive(Debug, Default, PartialEq)]
pub(crate) struct SourceProgramLine {
    location: Option<Location>,
    source_program_word: Option<SourceProgramWord>,
    comment: Option<Comment>
}

impl SourceProgramLine {
    pub(crate) fn new(
        location: Option<Location>,
        source_program_word: Option<SourceProgramWord>,
        comment: Option<Comment>) -> Self {
        Self { location, source_program_word, comment }
    }
}

pub(crate) type Location = Identifier;

pub(crate) type Identifier = String;

#[derive(Debug, PartialEq)]
pub(crate) enum SourceProgramWord {
    SWord(SWord),
    PWord(PWord),
    FWord(FWord),
    IWord(IWord),
    Octal(Octal),
}

pub(crate) type SWord = String;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PWord {
    TakeType(Mnemonic, Option<Acc>, GeneralOperand),
    PutType(Mnemonic, Option<Acc>, AddressOperand),
    LoadN(Option<Acc>, SimpleAddressOperand, Index),
    LoadRConst(Option<Acc>, ConstOperand, Index),
    LoadR(Option<Acc>, SimpleAddressOperand, Index),
    LibraryMnemonic(Mnemonic),
}

pub(crate) type Acc = u8;

pub(crate) type FWord = FloatType;

pub(crate) type IWord = IntType;

pub(crate) type Comment = String;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Octal {
    S(WordValue),
    P(WordValue),
    F(WordValue),
    I(WordValue),
}

impl std::fmt::Display for Octal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Octal::S(value) => write!(f, "(S{:08o})", value),
            Octal::P(value) => write!(f, "(P{:08o})", value),
            Octal::F(value) => write!(f, "(F{:08o})", value),
            Octal::I(value) => write!(f, "(I{:08o})", value),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GeneralOperand {
    AddressOperand(AddressOperand),
    ConstOperand(ConstOperand),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AddressOperand {
    address: SimpleAddressOperand,
    index: Option<Index>,
}

impl AddressOperand {
    pub(crate) fn new(
        address: SimpleAddressOperand,
        index:  Option<Index>) -> Self {
        Self { address, index }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ConstOperand {
    SignedInteger(IntType),
    SignedFWord(FloatType),
    Octal(Octal),
    SWord(SWord)
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SimpleAddressOperand {
    DirectAddress(Address),
    IndirectAddress(Address)
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Address {
    Identifier(String),
    NumericAddress(NumericAddress),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum NumericAddress {
    AbsoluteAddress(AddressRef),
    RelativeAddress(RelativeRef),
}

pub(crate) type TypeDesignator = u8;
pub(crate) type Character = u8;
pub(crate) type NumericCharacter = u8;
pub(crate) type Punctuation = u8;
pub(crate) type IntType = i32;
pub(crate) type FloatType = f32;
pub(crate) type AddressRef = usize;
pub(crate) type RelativeRef = usize; // TODO: Implies forward addressing only...
pub(crate) type Index = usize;
pub(crate) type WordValue = u32;

#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum Mnemonic {
    LDN,
    LDR,
    // 0-15 mnemonic
    NTHG,
    ADD,
    SUBT,
    MPLY,
    DVD,
    TAKE,
    NEG,
    MOD,
    CLR,
    AND,
    OR,
    NEQV,
    NOT,
    SHFR,
    CYCR,
    OPUT,
    // X 0-15 mnemonic
    XNTHG,
    XADD,
    XSUBT,
    XMPLY,
    XDVD,
    XTAKE,
    XNEG,
    XMOD,
    XCLR,
    XAND,
    XOR,
    XNEQV,
    XNOT,
    XSHFR,
    XCYCR,
    XOPUT,
    // 16-22 mnemonic
    IPUT,
    PUT,
    INCR,
    DECR,
    TYPE,
    CHYP,
    EXEC,
    // X 16-22 mnemonic
    XIPUT,
    XPUT,
    XINCR,
    XDECR,
    XTYPE,
    XCHYP,
    XEXEC,
    // skip mnemonic
    SKET,
    SKAE,
    SKAN,
    SKAL,
    SKAG,
    // jump mnemonic
    LIBR,
    JLIK,
    JUMP,
    JEZ,
    JNZ,
    JLZ,
    JGZ,
    JOI,
    SLIK,
    SNLZ,
    // library mnemonic
    SQRT,
    LN,
    EXP,
    READ,
    PRINT,
    SIN,
    COS,
    TAN,
    ARCTAN,
    STOP,
    LINE,
    INT,
    FRAC,
    FLOAT,
    CAPTN,
}
