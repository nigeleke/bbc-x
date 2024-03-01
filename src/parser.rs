use crate::ast::*;
use crate::grammar::*;
use crate::result::{Error, Result};

struct Parser;

impl Parser {
    fn parse(program: &str) -> Result<SourceProgram> {
        let lines = program.lines().enumerate();
        
        let non_empty_line =
            |(_, line): &(usize, &str)| !line.trim().is_empty();
        
        let parse_line =
            |(i, line): (usize, &str)| match Grammar::bbcx_line().parse(line.as_bytes()) {
                Ok(parsed_line) => Ok(parsed_line),
                Err(error) => Err(Error::InvalidLine(i+1, line.trim().to_string(), error.to_string()))
            };
        
        let (good, bad): (Vec<Result<_>>, Vec<Result<_>>) = lines
            .filter(non_empty_line)
            .map(parse_line)
            .partition(Result::is_ok);

        if bad.is_empty() {
            Ok(good.into_iter().filter_map(Result::ok).collect::<Vec<_>>())
        } else {
            Err(Error::InvalidInput(bad.into_iter().filter_map(Result::err).collect()))
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
        let expected = vec![];
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
            SourceProgramLine::new(None, None, Some("".into())),
            SourceProgramLine::new(None, None, Some(" Comment 1".into())),
            SourceProgramLine::new(None, Some(SourceProgramWord::SWord("    ".into())), None),
            SourceProgramLine::new(None, Some(SourceProgramWord::SWord("  C2".into())), Some(" Comment 2".into())),
            SourceProgramLine::new(Some("LABEL1".into()), None, None),
            SourceProgramLine::new(Some("LABEL2".into()), None, Some(" Comment 3".into())),
            SourceProgramLine::new(Some("LABEL3".into()), Some(SourceProgramWord::SWord("L3  ".into())), None),
            SourceProgramLine::new(Some("LABEL4".into()), Some(SourceProgramWord::SWord("L4C4".into())), Some(" Comment 4".into())),
        ];
        assert_eq!(program, expected);
    }

    #[test]
    fn sword() {
        let program = r#"   "TEXT"
"#;
        let program = Parser::parse(program).unwrap();
        let expected = vec![
            SourceProgramLine::new(None, Some(SourceProgramWord::SWord("TEXT".into())), None),
        ];
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
            PWord::TakeType(Mnemonic::ADD, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::ADD, Some(b'0'), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())), None))),
            PWord::TakeType(Mnemonic::SKAG, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR3".into())), None))),
            PWord::TakeType(Mnemonic::SKAG, Some(b'1'), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR4".into())), None))),
        ];
        let expected = pwords
            .iter()
            .map(|p| SourceProgramLine::new(None, Some(SourceProgramWord::PWord(p.clone())), None))
            .collect::<SourceProgram>();
        assert_eq!(program, expected)
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
            PWord::PutType(Mnemonic::XADD, Some(b'0'), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::INCR, Some(b'1'), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())), None)),
            PWord::PutType(Mnemonic::XINCR, Some(b'2'), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR3".into())), None)),
            PWord::PutType(Mnemonic::JUMP, Some(b'3'), 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR4".into())), None)),
        ];
        let expected = pwords
            .iter()
            .map(|p| SourceProgramLine::new(None, Some(SourceProgramWord::PWord(p.clone())), None))
            .collect::<SourceProgram>();
        assert_eq!(program, expected)
    }

    #[test]
    fn pword_loadn() {
        let program = r#"
        LDN         ADDR1(42)
        LDN 1       ADDR2(57)
"#;
        let program = Parser::parse(program).unwrap();
        let pwords = vec![
            PWord::LoadN(None, 
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), 42),
            PWord::LoadN(Some(b'1'), 
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())), 57),
        ];
        let expected = pwords
            .iter()
            .map(|p| SourceProgramLine::new(None, Some(SourceProgramWord::PWord(p.clone())), None))
            .collect::<SourceProgram>();
        assert_eq!(program, expected)
    }

    #[test]
    fn pword_loadr_const() {
        let program = r#"
        LDR         +24(42)
        LDR 1       -3.14(57)
"#;
        let program = Parser::parse(program).unwrap();
        let pwords = vec![
            PWord::LoadRConst(None, ConstOperand::SignedInteger(24), 42),
            PWord::LoadRConst(Some(b'1'), ConstOperand::SignedFWord(-3.14), 57),
        ];
        let expected = pwords
            .iter()
            .map(|p| SourceProgramLine::new(None, Some(SourceProgramWord::PWord(p.clone())), None))
            .collect::<SourceProgram>();
        assert_eq!(program, expected)
    }

    #[test]
    fn pword_loadr() {
        let program = r#"
        LDR         ADDR1(42)
        LDR 1       ADDR2(57)
"#;
        let program = Parser::parse(program).unwrap();
        let pwords = vec![
            PWord::LoadR(None, 
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), 42),
            PWord::LoadR(Some(b'1'), 
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())), 57),
        ];
        let expected = pwords
            .iter()
            .map(|p| SourceProgramLine::new(None, Some(SourceProgramWord::PWord(p.clone())), None))
            .collect::<SourceProgram>();
        assert_eq!(program, expected)
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
        assert_eq!(program, expected)
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
        assert_eq!(program, expected)
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
            PWord::TakeType(Mnemonic::ADD, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::ADD, Some(b'0'), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())), Some(42)))),
            PWord::TakeType(Mnemonic::ADD, Some(b'1'), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::IndirectAddress(Address::Identifier("ADDR3".into())), None))),
            PWord::TakeType(Mnemonic::ADD, Some(b'2'), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::IndirectAddress(Address::Identifier("ADDR4".into())), Some(42)))),
            PWord::TakeType(Mnemonic::ADD, Some(b'3'), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::NumericAddress(NumericAddress::AbsoluteAddress(512))), None))),
            PWord::TakeType(Mnemonic::ADD, Some(b'4'), 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::NumericAddress(NumericAddress::RelativeAddress(512))), None))),
            PWord::TakeType(Mnemonic::ADD, Some(b'5'), 
                GeneralOperand::ConstOperand(ConstOperand::SignedInteger(-42))),
            PWord::TakeType(Mnemonic::ADD, Some(b'6'), 
                GeneralOperand::ConstOperand(ConstOperand::SignedFWord(3.14))),
            PWord::TakeType(Mnemonic::ADD, Some(b'7'), 
                GeneralOperand::ConstOperand(ConstOperand::Octal(Octal::I(0o1234567)))),
            PWord::TakeType(Mnemonic::ADD, None, 
                GeneralOperand::ConstOperand(ConstOperand::SWord("TEXT".into()))),
        ];
        let expected = pwords
            .iter()
            .map(|p| SourceProgramLine::new(None, Some(SourceProgramWord::PWord(p.clone())), None))
            .collect::<SourceProgram>();
        assert_eq!(program, expected)
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
            PWord::LoadN(None, 
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), 42),
            PWord::LoadR(None, 
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), 42),
            PWord::TakeType(Mnemonic::NTHG, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::ADD, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::SUBT, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::MPLY, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::DVD, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::TAKE, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::NEG, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::MOD, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::CLR, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::AND, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::OR, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::NEQV, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::NOT, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::SHFR, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::CYCR, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::OPUT, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::PutType(Mnemonic::XNTHG, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XADD, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XSUBT, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XMPLY, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XDVD, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XTAKE, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XNEG, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XMOD, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XCLR, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XAND, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XOR, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XNEQV, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XNOT, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XSHFR, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XCYCR, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XOPUT, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::IPUT, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::PUT, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::INCR, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::DECR, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::TYPE, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::CHYP, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::EXEC, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XIPUT, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XPUT, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XINCR, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XDECR, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XTYPE, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XCHYP, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::XEXEC, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::TakeType(Mnemonic::SKET, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::SKAE, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::SKAN, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::SKAL, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::TakeType(Mnemonic::SKAG, None, 
                GeneralOperand::AddressOperand(AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None))),
            PWord::PutType(Mnemonic::LIBR, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::JLIK, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::JUMP, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::JEZ, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::JNZ, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::JLZ, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::JGZ, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::JOI, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::SLIK, None, 
                AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())), None)),
            PWord::PutType(Mnemonic::SNLZ, None, 
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
        
        assert_eq!(program.len(), expected.len());
        let results = program.iter().zip(expected.iter());
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
        let errors = Parser::parse(program).err().unwrap();
        let expected = Error::InvalidInput(vec![
            Error::InvalidLine(3, "Invalid One".into(), "failed to parse source_program_line at 0, (inner: Mismatch at 8: expect end of input, found: 73)".into()),
            Error::InvalidLine(7, "Invalid Two".into(), "failed to parse source_program_line at 0, (inner: Mismatch at 8: expect end of input, found: 73)".into())
        ]);
        assert_eq!(errors, expected)
    }
}
