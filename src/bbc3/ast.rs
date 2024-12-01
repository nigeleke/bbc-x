#[derive(Clone, Debug, PartialEq)]
pub struct SourceProgramLine {
    location: Location,
    source_program_word: SourceProgramWord,
    comment: Comment,
}

impl SourceProgramLine {
    pub fn new(
        location: Location,
        source_program_word: SourceProgramWord,
        comment: Comment,
    ) -> Self {
        Self {
            location,
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
        let source_program_word = self.source_program_word.to_string();
        let comment = self.comment.to_string();

        write!(f, "{:<8}{:<42}{}", location, source_program_word, comment)
    }
}

pub type Location = AddressRef;

pub type Identifier = String;

#[derive(Clone, Debug, PartialEq)]
pub enum SourceProgramWord {
    SWord(SWord),
    PWord(PWord),
    FWord(FWord),
    IWord(IWord),
    Octal(Octal),
}

impl std::fmt::Display for SourceProgramWord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            SourceProgramWord::SWord(sword) => write!(f, "<{}>", sword),
            SourceProgramWord::PWord(pword) => write!(f, "{}", pword),
            SourceProgramWord::FWord(fword) => write!(f, "{}", fword),
            SourceProgramWord::IWord(iword) => write!(f, "{}", iword),
            SourceProgramWord::Octal(octal) => write!(f, "{}", octal),
        }
    }
}

pub type SWord = String;

#[derive(Clone, Debug, PartialEq)]
pub enum PWord {
    TakeType(Mnemonic, Acc, GeneralOperand),
    PutType(Mnemonic, Acc, AddressOperand),
    LoadN(Acc, SimpleAddressOperand, Index),
    LoadRConst(Acc, ConstOperand, Index),
    LoadR(Acc, SimpleAddressOperand, Index),
    LibraryMnemonic(Mnemonic),
}

impl std::fmt::Display for PWord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            PWord::TakeType(inst, acc, operand) => write!(
                f,
                "{:<8}{:>2} {}",
                inst.to_string(),
                acc.to_string(),
                operand
            ),
            PWord::PutType(inst, acc, operand) => write!(
                f,
                "{:<8}{:>2} {}",
                inst.to_string(),
                acc.to_string(),
                operand
            ),
            PWord::LoadN(acc, operand, index) => write!(
                f,
                "{:<8}{:>2} {}:{}",
                Mnemonic::LDN.to_string(),
                acc.to_string(),
                operand,
                index
            ),
            PWord::LoadRConst(acc, operand, index) => write!(
                f,
                "{:<8}{:>2} {}:{}",
                Mnemonic::LDR.to_string(),
                acc.to_string(),
                operand,
                index
            ),
            PWord::LoadR(acc, operand, index) => write!(
                f,
                "{:<8}{:>2} {}:{}",
                Mnemonic::LDR.to_string(),
                acc.to_string(),
                operand,
                index
            ),
            PWord::LibraryMnemonic(inst) => write!(f, "{:<8}", inst.to_string()),
        }
    }
}

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

pub type Comment = String;

#[derive(Clone, Debug, PartialEq)]
pub enum Octal {
    S(WordValue),
    P(WordValue),
    F(WordValue),
    I(WordValue),
}

impl std::fmt::Display for Octal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Octal::S(value) => write!(f, "(S{:08o})", value),
            Octal::P(value) => write!(f, "(P{:08o})", value),
            Octal::F(value) => write!(f, "(F{:08o})", value),
            Octal::I(value) => write!(f, "(I{:08o})", value),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeneralOperand {
    AddressOperand(AddressOperand),
    ConstOperand(ConstOperand),
}

impl std::fmt::Display for GeneralOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            GeneralOperand::AddressOperand(o) => write!(f, "{}", o),
            GeneralOperand::ConstOperand(o) => write!(f, "{}", o),
        }
    }
}

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

#[derive(Clone, Debug, PartialEq)]
pub enum ConstOperand {
    SignedInteger(IntType),
    SignedFWord(FloatType),
    Octal(Octal),
    SWord(SWord),
}

impl std::fmt::Display for ConstOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            ConstOperand::SignedInteger(c) => write!(f, "{:+}", c),
            ConstOperand::SignedFWord(c) => write!(f, "{:+}", c),
            ConstOperand::Octal(c) => write!(f, "{}", c),
            ConstOperand::SWord(c) => write!(f, "<{}>", c),
        }
    }
}

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

#[derive(Clone, Debug, PartialEq)]
pub enum NumericAddress {
    AbsoluteAddress(AddressRef),
    RelativeAddress(RelativeRef),
}

impl std::fmt::Display for NumericAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            NumericAddress::AbsoluteAddress(a) => write!(f, "{}", a),
            NumericAddress::RelativeAddress(a) => write!(f, "{}+", a),
        }
    }
}

pub type TypeDesignator = char;
pub type Character = char;
pub type NumericCharacter = char;
pub type Punctuation = char;
pub type IntType = i32;
pub type FloatType = f32;
pub type AddressRef = usize;
pub type RelativeRef = usize; // TODO: Implies forward addressing only...
pub type Index = usize;
pub type WordValue = u32;

#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Mnemonic {
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

impl std::fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
