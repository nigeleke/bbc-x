use super::ast::*;
use super::grammar::*;

use crate::result::{Error, Result};

pub struct Parser;

impl Parser {
    pub fn parse_line(input: &str) -> Result<SourceProgramLine> {
        Grammar::bbc3_line()
            .parse(input.trim().as_bytes())
            .map_err(|_| Error::FailedToParse(input.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    type SourceProgram = Vec<Result<SourceProgramLine>>;

    fn parse(input: &str) -> SourceProgram {
        input
            .to_string()
            .lines()
            .map(Parser::parse_line)
            .collect::<SourceProgram>()
    }

    fn from_pwords(pwords: &[PWord]) -> SourceProgram {
        pwords
            .iter()
            .enumerate()
            .map(|(i, p)| {
                Ok(SourceProgramLine::new(
                    i + 1,
                    SourceProgramWord::PWord(p.clone()),
                    "".into(),
                ))
            })
            .collect::<SourceProgram>()
    }

    #[test]
    fn locations_and_comments() {
        let program = r#"
0001    <    >
0002    <  C1>      ; Comment 1
"#;
        let actual = parse(program);
        let expected = vec![
            Ok(SourceProgramLine::new(
                1,
                SourceProgramWord::SWord("    ".into()),
                "".into(),
            )),
            Ok(SourceProgramLine::new(
                2,
                SourceProgramWord::SWord("  C1".into()),
                "; Comment 1".into(),
            )),
        ]
        .into_iter()
        .collect::<SourceProgram>();
        assert_eq!(actual[1..], expected);
    }

    #[test]
    fn sword() {
        let program = r#"0001   <TEXT>
"#;
        let actual = parse(program);
        let expected = vec![Ok(SourceProgramLine::new(
            1,
            SourceProgramWord::SWord("TEXT".into()),
            "".into(),
        ))]
        .into_iter()
        .collect::<SourceProgram>();
        assert_eq!(actual, expected)
    }

    #[test]
    fn pword_take_type() {
        let program = r#"
0001    ADD     ADDR1
0002    ADD2    ADDR2
0003    SKAG    ADDR3
0004    SKAG2   ADDR4
"#;
        let actual = parse(program);
        let pwords = vec![
            PWord::TakeType(
                Mnemonic::ADD,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::ADD,
                '2'.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::SKAG,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR3".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::SKAG,
                '2'.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR4".into())),
                    None,
                )),
            ),
        ];
        let expected = from_pwords(&pwords);
        assert_eq!(actual[1..], expected)
    }

    #[test]
    fn pword_put_type() {
        let program = r#"
0001    XADD    ADDR1
0002    INCR    ADDR2
0003    XINCR2  ADDR3
0004    JUMP2   ADDR4
"#;
        let actual = parse(program);
        let pwords = vec![
            PWord::PutType(
                Mnemonic::XADD,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::INCR,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XINCR,
                '2'.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR3".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::JUMP,
                '2'.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR4".into())),
                    None,
                ),
            ),
        ];
        let expected = from_pwords(&pwords);
        assert_eq!(actual[1..], expected)
    }

    #[test]
    fn pword_loadn() {
        let program = r#"
0001    LDN     ADDR1:42
0002    LDN2    ADDR2:57
"#;
        let actual = parse(program);
        let pwords = vec![
            PWord::LoadN(
                None.into(),
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                42,
            ),
            PWord::LoadN(
                '2'.into(),
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())),
                57,
            ),
        ];
        let expected = from_pwords(&pwords);
        assert_eq!(actual[1..], expected)
    }

    #[test]
    fn pword_loadr_const() {
        let program = r#"
0001    LDR     +24:42
0002    LDR2    -3.14:57
"#;
        let actual = parse(program);
        let pwords = vec![
            PWord::LoadRConst(None.into(), ConstOperand::SignedInteger(24), 42),
            PWord::LoadRConst('2'.into(), ConstOperand::SignedFWord(-3.14), 57),
        ];
        let expected = from_pwords(&pwords);
        assert_eq!(actual[1..], expected)
    }

    #[test]
    fn pword_loadr() {
        let program = r#"
0001    LDR     ADDR1:42
0002    LDR2    ADDR2:57
"#;
        let actual = parse(program);
        let pwords = vec![
            PWord::LoadR(
                None.into(),
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                42,
            ),
            PWord::LoadR(
                '2'.into(),
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())),
                57,
            ),
        ];
        let expected = from_pwords(&pwords);
        assert_eq!(actual[1..], expected)
    }

    #[test]
    fn pword_library_mnemonic() {
        let program = r#"
0001    SQRT
0002    LN
0003    EXP
0004    READ
0005    PRINT
0006    SIN
0007    COS
0008    TAN
0009    ARCTAN
0010    STOP
0011    LINE
0012    INT
0013    FRAC
0014    FLOAT
0015    CAPTN
"#;
        let actual = parse(program);
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
        let expected = from_pwords(&pwords);
        assert_eq!(actual[1..], expected)
    }

    #[test]
    fn fword() {
        let program = r#"
0001   3.14
"#;
        let actual = parse(program);
        let expected = vec![Ok(SourceProgramLine::new(
            1,
            SourceProgramWord::FWord(3.14),
            "".into(),
        ))];
        assert_eq!(actual[1..], expected)
    }

    #[test]
    fn iword() {
        let program = r#"
0001   42
"#;
        let actual = parse(program);
        let expected = vec![Ok(SourceProgramLine::new(
            1,
            SourceProgramWord::IWord(42),
            "".into(),
        ))];
        assert_eq!(actual[1..], expected)
    }

    #[test]
    fn octal() {
        let program = r#"
0001    (S11110000)
0002    (P33332222)
0003    (F55554444)
0004    (I77776666)
"#;
        let actual = parse(program);
        let expected = vec![
            Ok(SourceProgramLine::new(
                1,
                SourceProgramWord::Octal(Octal::S(0o11110000)),
                "".into(),
            )),
            Ok(SourceProgramLine::new(
                2,
                SourceProgramWord::Octal(Octal::P(0o33332222)),
                "".into(),
            )),
            Ok(SourceProgramLine::new(
                3,
                SourceProgramWord::Octal(Octal::F(0o55554444)),
                "".into(),
            )),
            Ok(SourceProgramLine::new(
                4,
                SourceProgramWord::Octal(Octal::I(0o77776666)),
                "".into(),
            )),
        ];
        assert_eq!(actual[1..], expected)
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
0001    ADD     ADDR1
0002    ADD2    ADDR2:42
0003    ADD     *ADDR3
0004    ADD2    *ADDR4:42
0005    ADD     512
0006    ADD2    512+
0007    ADD     -42
0008    ADD2    +3.14
0009    ADD     (I01234567)
0010    ADD2    <TEXT>
"#;
        let actual = parse(program);
        let pwords = vec![
            PWord::TakeType(
                Mnemonic::ADD,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::ADD,
                '2'.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())),
                    Some(42),
                )),
            ),
            PWord::TakeType(
                Mnemonic::ADD,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::IndirectAddress(Address::Identifier("ADDR3".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::ADD,
                '2'.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::IndirectAddress(Address::Identifier("ADDR4".into())),
                    Some(42),
                )),
            ),
            PWord::TakeType(
                Mnemonic::ADD,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::NumericAddress(
                        NumericAddress::AbsoluteAddress(512),
                    )),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::ADD,
                '2'.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::NumericAddress(
                        NumericAddress::RelativeAddress(512),
                    )),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::ADD,
                None.into(),
                GeneralOperand::ConstOperand(ConstOperand::SignedInteger(-42)),
            ),
            PWord::TakeType(
                Mnemonic::ADD,
                '2'.into(),
                GeneralOperand::ConstOperand(ConstOperand::SignedFWord(3.14)),
            ),
            PWord::TakeType(
                Mnemonic::ADD,
                None.into(),
                GeneralOperand::ConstOperand(ConstOperand::Octal(Octal::I(0o1234567))),
            ),
            PWord::TakeType(
                Mnemonic::ADD,
                '2'.into(),
                GeneralOperand::ConstOperand(ConstOperand::SWord("TEXT".into())),
            ),
        ];
        let expected = from_pwords(&pwords);
        assert_eq!(actual[1..], expected)
    }

    #[test]
    fn mnemonics() {
        let program = r#"
0001    LDN         ADDR1:42
0002    LDR         ADDR1:42
0003    NTHG        ADDR1
0004    ADD         ADDR1
0005    SUBT        ADDR1
0006    MPLY        ADDR1
0007    DVD         ADDR1
0008    TAKE        ADDR1
0009    NEG         ADDR1
0010    MOD         ADDR1
0011    CLR         ADDR1
0012    AND         ADDR1
0013    OR          ADDR1
0014    NEQV        ADDR1
0015    NOT         ADDR1
0016    SHFR        ADDR1
0017    CYCR        ADDR1
0018    OPUT        ADDR1
0019    XNTHG       ADDR1
0020    XADD        ADDR1
0021    XSUBT       ADDR1
0022    XMPLY       ADDR1
0023    XDVD        ADDR1
0024    XTAKE       ADDR1
0025    XNEG        ADDR1
0026    XMOD        ADDR1
0027    XCLR        ADDR1
0028    XAND        ADDR1
0029    XOR         ADDR1
0030    XNEQV       ADDR1
0031    XNOT        ADDR1
0032    XSHFR       ADDR1
0033    XCYCR       ADDR1
0034    XOPUT       ADDR1
0035    IPUT        ADDR1
0036    PUT         ADDR1
0037    INCR        ADDR1
0038    DECR        ADDR1
0039    TYPE        ADDR1
0040    CHYP        ADDR1
0041    EXEC        ADDR1
0042    XIPUT       ADDR1
0043    XPUT        ADDR1
0044    XINCR       ADDR1
0045    XDECR       ADDR1
0046    XTYPE       ADDR1
0047    XCHYP       ADDR1
0048    XEXEC       ADDR1
0049    SKET        ADDR1
0050    SKAE        ADDR1
0051    SKAN        ADDR1
0052    SKAL        ADDR1
0053    SKAG        ADDR1
0054    LIBR        ADDR1
0055    JLIK        ADDR1
0056    JUMP        ADDR1
0057    JEZ         ADDR1
0058    JNZ         ADDR1
0059    JLZ         ADDR1
0060    JGZ         ADDR1
0061    JOI         ADDR1
0062    SLIK        ADDR1
0063    SNLZ        ADDR1
0064    SQRT
0065    LN
0066    EXP
0067    READ
0068    PRINT
0069    SIN
0070    COS
0071    TAN
0072    ARCTAN
0073    STOP
0074    LINE
0075    INT
0076    FRAC
0077    FLOAT
0078    CAPTN
"#;
        let actual = parse(program);
        let pwords = vec![
            PWord::LoadN(
                None.into(),
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                42,
            ),
            PWord::LoadR(
                None.into(),
                SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                42,
            ),
            PWord::TakeType(
                Mnemonic::NTHG,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::ADD,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::SUBT,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::MPLY,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::DVD,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::TAKE,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::NEG,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::MOD,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::CLR,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::AND,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::OR,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::NEQV,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::NOT,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::SHFR,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::CYCR,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::OPUT,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::PutType(
                Mnemonic::XNTHG,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XADD,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XSUBT,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XMPLY,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XDVD,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XTAKE,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XNEG,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XMOD,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XCLR,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XAND,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XOR,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XNEQV,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XNOT,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XSHFR,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XCYCR,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XOPUT,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::IPUT,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::PUT,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::INCR,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::DECR,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::TYPE,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::CHYP,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::EXEC,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XIPUT,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XPUT,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XINCR,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XDECR,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XTYPE,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XCHYP,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::XEXEC,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::TakeType(
                Mnemonic::SKET,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::SKAE,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::SKAN,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::SKAL,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::TakeType(
                Mnemonic::SKAG,
                None.into(),
                GeneralOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::PutType(
                Mnemonic::LIBR,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::JLIK,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::JUMP,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::JEZ,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::JNZ,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::JLZ,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::JGZ,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::JOI,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::SLIK,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
            PWord::PutType(
                Mnemonic::SNLZ,
                None.into(),
                AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                ),
            ),
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
        let expected = from_pwords(&pwords);
        assert_eq!(actual.len(), expected.len() + 1);
        let results = actual.iter().skip(1).zip(expected.iter());
        for (line, expected) in results {
            // Line by line assert easier to detect errors.
            assert_eq!(line, expected)
        }
    }

    #[test]
    fn invalid_syntax() {
        let program = r#"
0001
0002    Invalid One
0003
0004    SQRT        ; Valid
0005
0006    Invalid Two
"#;
        let actual = parse(program);
        let expected = vec![
            Err(Error::FailedToParse("0001".into())),
            Err(Error::FailedToParse("0002    Invalid One".into())),
            Err(Error::FailedToParse("0003".into())),
            Ok(SourceProgramLine::new(
                4,
                SourceProgramWord::PWord(PWord::LibraryMnemonic(Mnemonic::SQRT)),
                "; Valid".into(),
            )),
            Err(Error::FailedToParse("0005".into())),
            Err(Error::FailedToParse("0006    Invalid Two".into())),
        ];
        assert_eq!(actual[1..], expected)
    }
}
