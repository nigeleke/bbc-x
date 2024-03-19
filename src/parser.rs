use crate::ast::*;
use crate::grammar::*;
use crate::result::{Error, Result};

pub(crate) type LineNumber = usize;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ParsedLine {
    line_number: LineNumber,
    parse_result: Result<SourceProgramLine>,
}

impl ParsedLine {
    fn new(line_number: LineNumber, parse_result: Result<SourceProgramLine>) -> Self {
        ParsedLine { line_number, parse_result }
    }

    pub(crate) fn line_number(&self) -> LineNumber {
        self.line_number
    }

    pub(crate) fn parse_result(&self) -> &Result<SourceProgramLine> {
        &self.parse_result
    }
}

pub(crate) struct Parser;

impl Parser {
    pub(crate) fn parse(program: &str) -> Result<SourceProgram> {
        let lines = program.lines();
        let lines_count = lines.clone().count();
        let lines = lines.enumerate();
        
        let parse_line =
            |(i, line): (LineNumber, &str)|
                match Grammar::bbcx_line()
                    .parse(line.as_bytes()) {
                        Ok(line) => ParsedLine::new(i+1, Ok(line)),
                        Err(error) => ParsedLine::new(i+1, Err(Error::InvalidLine(line.trim().to_string(), error.to_string())))
                    };
        
        let all_results = lines.map(parse_line);
        let ok_results = all_results.clone()
            .filter_map(|l| l.parse_result.ok())
            .collect::<SourceProgram>();

        let all_ok = lines_count == ok_results.len();

        if all_ok {
            Ok(ok_results)
        } else {
            Err(Error::InvalidInput(all_results.collect()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn allow_empty_lines() {
        let program = r#"

"#;
        let program = Parser::parse(program).unwrap();
        let expected = vec![
            SourceProgramLine::default(),
            SourceProgramLine::default(),
        ];
        assert_eq!(program, expected);
    }

    #[test]
    fn labels_and_comments() {
        let program = r#"
                    ;
                    ; Comment 1
        "    "
        "  C2"      ; Comment 2
LABEL1:
LABEL2:             ; Comment 3
LABEL3: "L3  "
LABEL4: "L4C4"      ; Comment 4
"#;
        let program = Parser::parse(program).unwrap();
        let expected = vec![
            SourceProgramLine::default(),
            SourceProgramLine::new(None, None, Some("".into())),
            SourceProgramLine::new(None, None, Some(" Comment 1".into())),
            SourceProgramLine::new(None, Some(SourceProgramWord::SWord("    ".into())), None),
            SourceProgramLine::new(None, Some(SourceProgramWord::SWord("  C2".into())), Some(" Comment 2".into())),
            SourceProgramLine::new(Some("LABEL1".into()), None, None),
            SourceProgramLine::new(Some("LABEL2".into()), None, Some(" Comment 3".into())),
            SourceProgramLine::new(Some("LABEL3".into()), Some(SourceProgramWord::SWord("L3  ".into())), None),
            SourceProgramLine::new(Some("LABEL4".into()), Some(SourceProgramWord::SWord("L4C4".into())), Some(" Comment 4".into())),
        ].into_iter()
         .collect::<SourceProgram>();
        assert_eq!(program, expected);
    }

    #[test]
    fn sword() {
        let program = r#"   "TEXT"
"#;
        let program = Parser::parse(program).unwrap();
        let expected = vec![
            SourceProgramLine::new(None, Some(SourceProgramWord::SWord("TEXT".into())), None),
        ].into_iter()
         .collect::<SourceProgram>();
        assert_eq!(program, expected)
    }

    #[test]
    fn pword_take_type() {
        let program = r#"
        ADD         ADDR1
        ADD 0       ADDR2
        SKAG        ADDR3
        SKAG 1      ADDR4
"#;
        let program = Parser::parse(program).unwrap();
        let pwords = vec![
            PWord::TakeType(Mnemonic::ADD, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::ADD, b'0'.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())), None))),
            PWord::TakeType(Mnemonic::SKAG, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR3".into())), None))),
            PWord::TakeType(Mnemonic::SKAG, b'1'.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR4".into())), None))),
        ];
        let expected = pwords
            .iter()
            .map(|p| SourceProgramLine::new(None, Some(SourceProgramWord::PWord(p.clone())), None))
            .collect::<SourceProgram>();
        assert_eq!(program[1..], expected)
    }

    #[test]
    fn pword_put_type() {
        let program = r#"
        XADD  0     ADDR1
        INCR  1     ADDR2
        XINCR 2     ADDR3
        JUMP  3     ADDR4
"#;
        let program = Parser::parse(program).unwrap();
        let pwords = vec![
            PWord::PutType(Mnemonic::XADD, b'0'.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::INCR, b'1'.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())), None)),
            PWord::PutType(Mnemonic::XINCR, b'2'.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR3".into())), None)),
            PWord::PutType(Mnemonic::JUMP, b'3'.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR4".into())), None)),
        ];
        let expected = pwords
            .iter()
            .map(|p| SourceProgramLine::new(None, Some(SourceProgramWord::PWord(p.clone())), None))
            .collect::<SourceProgram>();
        assert_eq!(program[1..], expected)
    }

    #[test]
    fn pword_loadn() {
        let program = r#"
        LDN         ADDR1(42)
        LDN 1       ADDR2(57)
"#;
        let program = Parser::parse(program).unwrap();
        let pwords = vec![
            PWord::LoadN(None.into(), 
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), 42),
            PWord::LoadN(b'1'.into(), 
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())), 57),
        ];
        let expected = pwords
            .iter()
            .map(|p| SourceProgramLine::new(None, Some(SourceProgramWord::PWord(p.clone())), None))
            .collect::<SourceProgram>();
        assert_eq!(program[1..], expected)
    }

    #[test]
    fn pword_loadr_const() {
        let program = r#"
        LDR         +24(42)
        LDR 1       -3.14(57)
"#;
        let program = Parser::parse(program).unwrap();
        let pwords = vec![
            PWord::LoadRConst(None.into(), ConstOperand::SignedInteger(24), 42),
            PWord::LoadRConst(b'1'.into(), ConstOperand::SignedFWord(-3.14), 57),
        ];
        let expected = pwords
            .iter()
            .map(|p| SourceProgramLine::new(None, Some(SourceProgramWord::PWord(p.clone())), None))
            .collect::<SourceProgram>();
        assert_eq!(program[1..], expected)
    }

    #[test]
    fn pword_loadr() {
        let program = r#"
        LDR         ADDR1(42)
        LDR 1       ADDR2(57)
"#;
        let program = Parser::parse(program).unwrap();
        let pwords = vec![
            PWord::LoadR(None.into(), 
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), 42),
            PWord::LoadR(b'1'.into(), 
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())), 57),
        ];
        let expected = pwords
            .iter()
            .map(|p| SourceProgramLine::new(None, Some(SourceProgramWord::PWord(p.clone())), None))
            .collect::<SourceProgram>();
        assert_eq!(program[1..], expected)
    }

    #[test]
    fn pword_library_mnemonic() {
        let program = r#"
        SQRT
        LN
        EXP
        READ
        PRINT
        SIN
        COS
        TAN
        ARCTAN
        STOP
        LINE
        INT
        FRAC
        FLOAT
        CAPTN
"#;
        let program = Parser::parse(program).unwrap();
        let pwords = vec![
            PWord::LibraryMnemonic(Mnemonic::SQRT), 
            PWord::LibraryMnemonic(Mnemonic::LN),
            PWord::LibraryMnemonic(Mnemonic::EXP),
            PWord::LibraryMnemonic(Mnemonic::READ),
            PWord::LibraryMnemonic(Mnemonic::PRINT),
            PWord::LibraryMnemonic(Mnemonic::SIN),
            PWord::LibraryMnemonic(Mnemonic::COS),
            PWord::LibraryMnemonic(Mnemonic::TAN),
            PWord::LibraryMnemonic(Mnemonic::ARCTAN),
            PWord::LibraryMnemonic(Mnemonic::STOP),
            PWord::LibraryMnemonic(Mnemonic::LINE),
            PWord::LibraryMnemonic(Mnemonic::INT),
            PWord::LibraryMnemonic(Mnemonic::FRAC),
            PWord::LibraryMnemonic(Mnemonic::FLOAT),
            PWord::LibraryMnemonic(Mnemonic::CAPTN),
        ];
        let expected = pwords
            .iter()
            .map(|p| SourceProgramLine::new(None, Some(SourceProgramWord::PWord(p.clone())), None))
            .collect::<SourceProgram>();
        assert_eq!(program[1..], expected)
    }

    #[test]
    fn fword() {
        let program = r#"   3.14
"#;
        let program = Parser::parse(program).unwrap();
        let expected = vec![
            SourceProgramLine::new(None, Some(SourceProgramWord::FWord(3.14)), None)
        ];
        assert_eq!(program, expected)
    }

    #[test]
    fn iword() {
        let program = r#"   42
"#;
        let program = Parser::parse(program).unwrap();
        let expected = vec![
            SourceProgramLine::new(None, Some(SourceProgramWord::IWord(42)), None)
        ];
        assert_eq!(program, expected)
    }

    #[test]
    fn octal() {
        let program = r#"
        (S11110000)
        (P33332222)
        (F55554444)
        (I77776666)
"#;
        let program = Parser::parse(program).unwrap();
        let expected = vec![
            SourceProgramLine::new(None, Some(SourceProgramWord::Octal(Octal::S(0o11110000))), None),
            SourceProgramLine::new(None, Some(SourceProgramWord::Octal(Octal::P(0o33332222))), None),
            SourceProgramLine::new(None, Some(SourceProgramWord::Octal(Octal::F(0o55554444))), None),
            SourceProgramLine::new(None, Some(SourceProgramWord::Octal(Octal::I(0o77776666))), None),
        ];
        assert_eq!(program[1..], expected)
    }

    #[test]
    fn octal_display() {
        assert_eq!(format!("{}", Octal::S(0o11110000)), "(S11110000)");
        assert_eq!(format!("{}", Octal::P(0o33332222)), "(P33332222)");
        assert_eq!(format!("{}", Octal::F(0o55554444)), "(F55554444)");
        assert_eq!(format!("{}", Octal::I(0o77776666)), "(I77776666)")
    }

    #[test]
    fn addressing_modes() {
        let program = r#"
        ADD         ADDR1
        ADD 0       ADDR2(42)
        ADD 1       *ADDR3
        ADD 2       *ADDR4(42)
        ADD 3       512
        ADD 4       512+
        ADD 5       -42
        ADD 6       +3.14
        ADD 7       (I01234567)
        ADD         "TEXT"
"#;
        let program = Parser::parse(program).unwrap();
        let pwords = vec![
            PWord::TakeType(Mnemonic::ADD, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::ADD, b'0'.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())), Some(42)))),
            PWord::TakeType(Mnemonic::ADD, b'1'.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::IndirectAddress(Address::Identifier("ADDR3".into())), None))),
            PWord::TakeType(Mnemonic::ADD, b'2'.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::IndirectAddress(Address::Identifier("ADDR4".into())), Some(42)))),
            PWord::TakeType(Mnemonic::ADD, b'3'.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::NumericAddress(NumericAddress::AbsoluteAddress(512))), None))),
            PWord::TakeType(Mnemonic::ADD, b'4'.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::NumericAddress(NumericAddress::RelativeAddress(512))), None))),
            PWord::TakeType(Mnemonic::ADD, b'5'.into(), 
                GeneralOperand::ConstOperand(ConstOperand::SignedInteger(-42))),
            PWord::TakeType(Mnemonic::ADD, b'6'.into(), 
                GeneralOperand::ConstOperand(ConstOperand::SignedFWord(3.14))),
            PWord::TakeType(Mnemonic::ADD, b'7'.into(), 
                GeneralOperand::ConstOperand(ConstOperand::Octal(Octal::I(0o1234567)))),
            PWord::TakeType(Mnemonic::ADD, None.into(), 
                GeneralOperand::ConstOperand(ConstOperand::SWord("TEXT".into()))),
        ];
        let expected = pwords
            .iter()
            .map(|p| SourceProgramLine::new(None, Some(SourceProgramWord::PWord(p.clone())), None))
            .collect::<SourceProgram>();
        assert_eq!(program[1..], expected)
    }

    #[test]
    fn mnemonics() {
        let program = r#"
        LDN         ADDR1(42)
        LDR         ADDR1(42)
        NTHG        ADDR1
        ADD         ADDR1
        SUBT        ADDR1
        MPLY        ADDR1
        DVD         ADDR1
        TAKE        ADDR1
        NEG         ADDR1
        MOD         ADDR1
        CLR         ADDR1
        AND         ADDR1
        OR          ADDR1
        NEQV        ADDR1
        NOT         ADDR1
        SHFR        ADDR1
        CYCR        ADDR1
        OPUT        ADDR1
        XNTHG       ADDR1
        XADD        ADDR1
        XSUBT       ADDR1
        XMPLY       ADDR1
        XDVD        ADDR1
        XTAKE       ADDR1
        XNEG        ADDR1
        XMOD        ADDR1
        XCLR        ADDR1
        XAND        ADDR1
        XOR         ADDR1
        XNEQV       ADDR1
        XNOT        ADDR1
        XSHFR       ADDR1
        XCYCR       ADDR1
        XOPUT       ADDR1
        IPUT        ADDR1
        PUT         ADDR1
        INCR        ADDR1
        DECR        ADDR1
        TYPE        ADDR1
        CHYP        ADDR1
        EXEC        ADDR1
        XIPUT       ADDR1
        XPUT        ADDR1
        XINCR       ADDR1
        XDECR       ADDR1
        XTYPE       ADDR1
        XCHYP       ADDR1
        XEXEC       ADDR1
        SKET        ADDR1
        SKAE        ADDR1
        SKAN        ADDR1
        SKAL        ADDR1
        SKAG        ADDR1
        LIBR        ADDR1
        JLIK        ADDR1
        JUMP        ADDR1
        JEZ         ADDR1
        JNZ         ADDR1
        JLZ         ADDR1
        JGZ         ADDR1
        JOI         ADDR1
        SLIK        ADDR1
        SNLZ        ADDR1
        SQRT
        LN
        EXP
        READ
        PRINT
        SIN
        COS
        TAN
        ARCTAN
        STOP
        LINE
        INT
        FRAC
        FLOAT
        CAPTN
"#;        
        let program = Parser::parse(program).unwrap();
        let pwords = vec![
            PWord::LoadN(None.into(), 
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), 42),
            PWord::LoadR(None.into(), 
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), 42),
            PWord::TakeType(Mnemonic::NTHG, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::ADD, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::SUBT, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::MPLY, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::DVD, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::TAKE, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::NEG, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::MOD, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::CLR, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::AND, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::OR, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::NEQV, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::NOT, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::SHFR, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::CYCR, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::OPUT, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::PutType(Mnemonic::XNTHG, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XADD, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XSUBT, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XMPLY, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XDVD, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XTAKE, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XNEG, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XMOD, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XCLR, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XAND, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XOR, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XNEQV, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XNOT, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XSHFR, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XCYCR, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XOPUT, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::IPUT, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::PUT, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::INCR, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::DECR, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::TYPE, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::CHYP, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::EXEC, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XIPUT, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XPUT, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XINCR, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XDECR, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XTYPE, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XCHYP, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XEXEC, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::TakeType(Mnemonic::SKET, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::SKAE, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::SKAN, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::SKAL, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::SKAG, None.into(), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::PutType(Mnemonic::LIBR, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::JLIK, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::JUMP, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::JEZ, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::JNZ, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::JLZ, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::JGZ, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::JOI, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::SLIK, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::SNLZ, None.into(), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::LibraryMnemonic(Mnemonic::SQRT), 
            PWord::LibraryMnemonic(Mnemonic::LN),
            PWord::LibraryMnemonic(Mnemonic::EXP),
            PWord::LibraryMnemonic(Mnemonic::READ),
            PWord::LibraryMnemonic(Mnemonic::PRINT),
            PWord::LibraryMnemonic(Mnemonic::SIN),
            PWord::LibraryMnemonic(Mnemonic::COS),
            PWord::LibraryMnemonic(Mnemonic::TAN),
            PWord::LibraryMnemonic(Mnemonic::ARCTAN),
            PWord::LibraryMnemonic(Mnemonic::STOP),
            PWord::LibraryMnemonic(Mnemonic::LINE),
            PWord::LibraryMnemonic(Mnemonic::INT),
            PWord::LibraryMnemonic(Mnemonic::FRAC),
            PWord::LibraryMnemonic(Mnemonic::FLOAT),
            PWord::LibraryMnemonic(Mnemonic::CAPTN),

        ];
        let expected = pwords
            .iter()
            .map(|p| SourceProgramLine::new(None, Some(SourceProgramWord::PWord(p.clone())), None))
            .collect::<SourceProgram>();
        
        assert_eq!(program.len(), expected.len() + 1);
        let results = program.iter().skip(1).zip(expected.iter());
        for (line, expected) in results { // Line by line assert easier to detect errors.
            assert_eq!(line, expected)
        }
    }

    #[test]
    fn invalid_syntax() {
        let program = r#"

        Invalid One

        SQRT        ; Valid

        Invalid Two
"#;
        let results = Parser::parse(program).err().unwrap();
        let expected = Error::InvalidInput(vec![
            ParsedLine::new(1, Ok(SourceProgramLine::default())),
            ParsedLine::new(2, Ok(SourceProgramLine::default())),
            ParsedLine::new(3, Err(Error::InvalidLine("Invalid One".into(), "failed to parse source_program_line at 0, (inner: Mismatch at 8: expect end of input, found: 73)".into()))),
            ParsedLine::new(4, Ok(SourceProgramLine::default())),
            ParsedLine::new(5, Ok(SourceProgramLine::new(None, Some(SourceProgramWord::PWord(PWord::LibraryMnemonic(Mnemonic::SQRT))), Some(" Valid".into())))),
            ParsedLine::new(6, Ok(SourceProgramLine::default())),
            ParsedLine::new(7, Err(Error::InvalidLine("Invalid Two".into(), "failed to parse source_program_line at 0, (inner: Mismatch at 8: expect end of input, found: 73)".into()))),
        ]);
        assert_eq!(results, expected)
    }
}
