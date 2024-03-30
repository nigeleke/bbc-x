#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SourceProgramLine {
    location: Location,
    source_program_word: SourceProgramWord,
    comment: Comment
}

impl SourceProgramLine {
    pub(crate) fn new(
        location: Location,
        source_program_word: SourceProgramWord,
        comment: Comment) -> Self {
        Self { location, source_program_word, comment }
    }

    pub(crate) fn location(&self) -> &Location {
        &self.location
    }

    pub(crate) fn source_program_word(&self) -> &SourceProgramWord {
        &self.source_program_word
    }
}

impl std::fmt::Display for SourceProgramLine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        let location = self.location.to_string();
        let source_program_word = self.source_program_word.to_string();
        let comment = self.comment.to_string();

        write!(f, "{:<8}{:<42}{}", location, source_program_word, comment)
    }
}

pub(crate) type Location = AddressRef;

pub(crate) type Identifier = String;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SourceProgramWord {
    SWord(SWord),
    PWord(PWord),
    FWord(FWord),
    IWord(IWord),
    Octal(Octal),
}

impl std::fmt::Display for SourceProgramWord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            SourceProgramWord::SWord(sword) => write!(f, "\"{}\"", sword),
            SourceProgramWord::PWord(pword) => write!(f, "{}", pword),
            SourceProgramWord::FWord(fword) => write!(f, "{}", fword),
            SourceProgramWord::IWord(iword) => write!(f, "{}", iword),
            SourceProgramWord::Octal(octal) => write!(f, "{}", octal),
        }
    }
}

pub(crate) type SWord = String;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PWord {
    TakeType(Mnemonic, Acc, GeneralOperand),
    PutType(Mnemonic, Acc, AddressOperand),
    LoadN(Acc, SimpleAddressOperand, Index),
    LoadRConst(Acc, ConstOperand, Index),
    LoadR(Acc, SimpleAddressOperand, Index),
    LibraryMnemonic(Mnemonic),
}

impl PWord {
    pub(crate) fn identifier(&self) -> Option<Identifier> {
        match self {
            PWord::TakeType(_, _, operand) => operand.identifier(),
            PWord::PutType(_, _, operand) => operand.identifier(),
            PWord::LoadN(_, operand, _) => operand.identifier(),
            PWord::LoadRConst(_, _, _) => None,
            PWord::LoadR(_, operand, _) => operand.identifier(),
            PWord::LibraryMnemonic(_) => None,
        }        
    }
}

impl std::fmt::Display for PWord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            PWord::TakeType(inst, acc, operand) => write!(f, "{:<8}{:>2} {}", inst.to_string(), acc.to_string(), operand.to_string()),
            PWord::PutType(inst, acc, operand) => write!(f, "{:<8}{:>2} {}", inst.to_string(), acc.to_string(), operand.to_string()),
            PWord::LoadN(acc, operand, index) => write!(f, "{:<8}{:>2} {}({})", Mnemonic::LDN.to_string(), acc.to_string(), operand.to_string(), index.to_string()),
            PWord::LoadRConst(acc, operand, index) => write!(f, "{:<8}{:>2} {}({})", Mnemonic::LDR.to_string(), acc.to_string(), operand.to_string(), index.to_string()),
            PWord::LoadR(acc, operand, index) => write!(f, "{:<8}{:>2} {}({})", Mnemonic::LDR.to_string(), acc.to_string(), operand.to_string(), index.to_string()),
            PWord::LibraryMnemonic(inst) => write!(f, "{:<8}", inst.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Acc(Option<char>);

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
        write!(f, "{}", self.0.map_or("".into(), |a| (a as char).to_string()))
    }
}

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
pub(crate) enum GeneralOperand {
    AddressOperand(AddressOperand),
    ConstOperand(ConstOperand),
}

impl GeneralOperand {
    pub(crate) fn identifier(&self) -> Option<Identifier> {
        match self {
            GeneralOperand::AddressOperand(ao) => ao.identifier(),
            GeneralOperand::ConstOperand(_) => None,
        }        
    }
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

    pub(crate) fn identifier(&self) -> Option<Identifier> {
        self.address.identifier()
    }
}

impl std::fmt::Display for AddressOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        let index = self.index.map(|i| format!("({})", i)).unwrap_or("".into());
        write!(f, "{}{}", self.address, index)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ConstOperand {
    SignedInteger(IntType),
    SignedFWord(FloatType),
    Octal(Octal),
    SWord(SWord)
}

impl std::fmt::Display for ConstOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            ConstOperand::SignedInteger(c) => write!(f, "{:+}", c),
            ConstOperand::SignedFWord(c) => write!(f, "{:+}", c), 
            ConstOperand::Octal(c) => write!(f, "{}", c),
            ConstOperand::SWord(c) => write!(f, "\"{}\"", c),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SimpleAddressOperand {
    DirectAddress(Address),
    IndirectAddress(Address)
}

impl SimpleAddressOperand {
    pub(crate) fn identifier(&self) -> Option<Identifier> {
        match self {
            SimpleAddressOperand::DirectAddress(a) => a.identifier(),
            SimpleAddressOperand::IndirectAddress(a) => a.identifier(),
        }        
    }
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
pub(crate) enum Address {
    Identifier(Identifier),
    NumericAddress(NumericAddress),
}

impl Address {
    pub(crate) fn identifier(&self) -> Option<Identifier> {
        match self {
            Address::Identifier(i) => Some(i.into()),
            Address::NumericAddress(_) => None,
        }        
    }
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
pub(crate) enum NumericAddress {
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

pub(crate) type TypeDesignator = char;
pub(crate) type Character = char;
pub(crate) type NumericCharacter = char;
pub(crate) type Punctuation = char;
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

impl std::fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
