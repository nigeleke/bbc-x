use super::ast::*;
use super::grammar::*;

use crate::result::{Error, Result};

pub struct Parser;

impl Parser {
    pub fn parse_line(input: &str) -> Result<SourceProgramLine> {
        Grammar::bbcx_line()
            .parse(input.trim().as_bytes())
            .map_err(|_| Error::FailedToParse(input.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
                    Label::from(None),
                    SourceProgramWord::PWord(p.clone()),
                    "".into(),
                ))
            })
            .collect::<SourceProgram>()
    }

    #[test]
    fn locations_labels_and_comments() {
        let program = r#"
0001                "    "
0002                "  C1"      ; Comment 1
0003    LABEL1:     "    "
0004    LABEL2:     "  C2"      ; Comment 2
"#;
        let actual = parse(program);
        let expected = vec![
            Ok(SourceProgramLine::new(
                1,
                Label::from(None),
                SourceProgramWord::SWord("    ".into()),
                "".into(),
            )),
            Ok(SourceProgramLine::new(
                2,
                Label::from(None),
                SourceProgramWord::SWord("  C1".into()),
                "; Comment 1".into(),
            )),
            Ok(SourceProgramLine::new(
                3,
                Label::from("LABEL1".to_string()),
                SourceProgramWord::SWord("    ".into()),
                "".into(),
            )),
            Ok(SourceProgramLine::new(
                4,
                Label::from("LABEL2".to_string()),
                SourceProgramWord::SWord("  C2".into()),
                "; Comment 2".into(),
            )),
        ]
        .into_iter()
        .collect::<SourceProgram>();
        assert_eq!(actual[1..], expected);
    }

    #[test]
    fn sword() {
        let program = r#"0001   "TEXT"
"#;
        let actual = parse(program);
        let expected = vec![Ok(SourceProgramLine::new(
            1,
            Label::from(None),
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
0002    ADD 2,  ADDR2
0003    SKAG    ADDR3
0004    SKAG 2, ADDR4
"#;
        let actual = parse(program);
        let pwords = vec![
            PWord::new(
                Mnemonic::ADD,
                None.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::new(
                Mnemonic::ADD,
                '2'.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())),
                    None,
                )),
            ),
            PWord::new(
                Mnemonic::SKAG,
                None.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR3".into())),
                    None,
                )),
            ),
            PWord::new(
                Mnemonic::SKAG,
                '2'.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
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
0001    ADDX    ADDR1
0002    INCR    ADDR2
0003    JUMP2,   ADDR4
"#;
        let actual = parse(program);
        let pwords = vec![
            PWord::new(
                Mnemonic::ADDX,
                None.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::new(
                Mnemonic::INCR,
                None.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())),
                    None,
                )),
            ),
            PWord::new(
                Mnemonic::JUMP,
                '2'.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR4".into())),
                    None,
                )),
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
0009    ATN
0010    STOP
0011    LINE
0012    INT
0013    FRAC
0014    FLOAT
0015    CAPTN
0016    PAGE
0017    RND
0018    ABS
"#;
        let actual = parse(program);
        let pwords = vec![
            PWord::new(Mnemonic::SQRT, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::LN, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::EXP, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::READ, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::PRINT, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::SIN, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::COS, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::TAN, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::ATN, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::STOP, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::LINE, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::INT, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::FRAC, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::FLOAT, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::CAPTN, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::PAGE, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::RND, None.into(), StoreOperand::None),
            PWord::new(Mnemonic::ABS, None.into(), StoreOperand::None),
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
            None.into(),
            SourceProgramWord::FWord(3.14),
            "".into(),
        ))];
        assert_eq!(actual[1..], expected)
    }

    #[test]
    fn iword() {
        let program = r#"
0001   +42
"#;
        let actual = parse(program);
        let expected = vec![Ok(SourceProgramLine::new(
            1,
            None.into(),
            SourceProgramWord::IWord(42),
            "".into(),
        ))];
        assert_eq!(actual[1..], expected)
    }

    #[test]
    fn addressing_modes() {
        let program = r#"
0001    ADD     ADDR1
0002    ADD2,   ADDR2[42]
0003    ADD     *ADDR3
0004    ADD2,   *ADDR4[42]
0005    ADD     512
0006    ADD     -42
0007    ADD2,   +3.14
0008    ADD2,   "TEXT"
"#;
        let actual = parse(program);
        let pwords = vec![
            PWord::new(
                Mnemonic::ADD,
                None.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                    None,
                )),
            ),
            PWord::new(
                Mnemonic::ADD,
                '2'.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR2".into())),
                    Some(42),
                )),
            ),
            PWord::new(
                Mnemonic::ADD,
                None.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::IndirectAddress(Address::Identifier("ADDR3".into())),
                    None,
                )),
            ),
            PWord::new(
                Mnemonic::ADD,
                '2'.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::IndirectAddress(Address::Identifier("ADDR4".into())),
                    Some(42),
                )),
            ),
            PWord::new(
                Mnemonic::ADD,
                None.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::NumericAddress(512)),
                    None,
                )),
            ),
            PWord::new(
                Mnemonic::ADD,
                None.into(),
                StoreOperand::ConstOperand(ConstOperand::SignedInteger(-42)),
            ),
            PWord::new(
                Mnemonic::ADD,
                '2'.into(),
                StoreOperand::ConstOperand(ConstOperand::SignedFWord(3.14)),
            ),
            PWord::new(
                Mnemonic::ADD,
                '2'.into(),
                StoreOperand::ConstOperand(ConstOperand::SWord("TEXT".into())),
            ),
        ];
        let expected = from_pwords(&pwords);
        assert_eq!(actual[1..], expected)
    }

    #[test]
    fn mnemonics() {
        let program = r#"
0001    NIL         ADDR1
0002    NTHG        ADDR1
0003    OR          ADDR1
0004    NEQV        ADDR1
0005    AND         ADDR1
0006    ADD         ADDR1
0007    SUBT        ADDR1
0008    MULT        ADDR1
0009    MPLY        ADDR1
0010    DVD         ADDR1
0011    TAKE        ADDR1
0012    TSTR        ADDR1
0013    TNEG        ADDR1
0014    TNOT        ADDR1
0015    TTYP        ADDR1
0016    TTYZ        ADDR1
0017    TTTT        ADDR1
0018    TOUT        ADDR1
0019    SKIP        ADDR1
0020    SKAE        ADDR1
0021    SKAN        ADDR1
0022    SKET        ADDR1
0023    SKAL        ADDR1
0024    SKAG        ADDR1
0025    SKED        ADDR1
0026    SKEI        ADDR1
0027    SHL         ADDR1
0028    ROT         ADDR1
0029    DSHL        ADDR1
0030    DROT        ADDR1
0031    POWR        ADDR1
0032    DMULT       ADDR1
0033    DIV         ADDR1
0034    DDIV        ADDR1
0035    NILX        ADDR1
0036    SWAP        ADDR1
0037    ORX         ADDR1
0038    NEQVX       ADDR1
0039    ANDX        ADDR1
0040    ADDX        ADDR1
0041    SUBTX       ADDR1
0042    MULTX       ADDR1
0043    MPLYX       ADDR1
0044    DVDX        ADDR1
0045    PUT         ADDR1
0046    PSQU        ADDR1
0047    PNEG        ADDR1
0048    PNOT        ADDR1
0049    PTYP        ADDR1
0050    PTYZ        ADDR1
0051    PFFP        ADDR1
0052    PIN         ADDR1
0053    JUMP        ADDR1
0054    JEZ         ADDR1
0055    JNZ         ADDR1
0056    JAT         ADDR1
0057    JLZ         ADDR1
0058    JGZ         ADDR1
0059    JZD         ADDR1
0060    JZI         ADDR1
0061    DECR        ADDR1
0062    INCR        ADDR1
0063    MOCKP       ADDR1
0064    MOCKS       ADDR1
0065    DBYTE       ADDR1
0066    EXEC        ADDR1
0067    EXTRA       ADDR1
0068    SQRT
0069    LN
0070    EXP
0071    READ
0072    PRINT
0073    SIN
0074    COS
0075    TAN
0076    ATN
0077    STOP
0078    LINE
0079    INT
0080    FRAC
0081    FLOAT
0082    CAPTN
0083    PAGE
0084    RND
0085    ABS
"#;
        let actual = parse(program);
        let store_operand_mnemonics = vec![
            Mnemonic::NIL,
            Mnemonic::NIL,
            Mnemonic::OR,
            Mnemonic::NEQV,
            Mnemonic::AND,
            Mnemonic::ADD,
            Mnemonic::SUBT,
            Mnemonic::MULT,
            Mnemonic::MULT,
            Mnemonic::DVD,
            Mnemonic::TAKE,
            Mnemonic::TSTR,
            Mnemonic::TNEG,
            Mnemonic::TNOT,
            Mnemonic::TTYP,
            Mnemonic::TTYZ,
            Mnemonic::TTTT,
            Mnemonic::TOUT,
            Mnemonic::SKIP,
            Mnemonic::SKAE,
            Mnemonic::SKAN,
            Mnemonic::SKET,
            Mnemonic::SKAL,
            Mnemonic::SKAG,
            Mnemonic::SKED,
            Mnemonic::SKEI,
            Mnemonic::SHL,
            Mnemonic::ROT,
            Mnemonic::DSHL,
            Mnemonic::DROT,
            Mnemonic::POWR,
            Mnemonic::DMULT,
            Mnemonic::DIV,
            Mnemonic::DDIV,
            Mnemonic::NILX,
            Mnemonic::NILX,
            Mnemonic::ORX,
            Mnemonic::NEQVX,
            Mnemonic::ANDX,
            Mnemonic::ADDX,
            Mnemonic::SUBTX,
            Mnemonic::MULTX,
            Mnemonic::MULTX,
            Mnemonic::DVDX,
            Mnemonic::PUT,
            Mnemonic::PSQU,
            Mnemonic::PNEG,
            Mnemonic::PNOT,
            Mnemonic::PTYP,
            Mnemonic::PTYZ,
            Mnemonic::PFFP,
            Mnemonic::PIN,
            Mnemonic::JUMP,
            Mnemonic::JEZ,
            Mnemonic::JNZ,
            Mnemonic::JAT,
            Mnemonic::JLZ,
            Mnemonic::JGZ,
            Mnemonic::JZD,
            Mnemonic::JZI,
            Mnemonic::DECR,
            Mnemonic::INCR,
            Mnemonic::MOCKP,
            Mnemonic::MOCKS,
            Mnemonic::DBYTE,
            Mnemonic::EXEC,
            Mnemonic::EXTRA,
        ];
        let mut store_operand_pwords = store_operand_mnemonics
            .into_iter()
            .map(|m| {
                PWord::new(
                    m,
                    None.into(),
                    StoreOperand::AddressOperand(AddressOperand::new(
                        SimpleAddressOperand::DirectAddress(Address::Identifier("ADDR1".into())),
                        None,
                    )),
                )
            })
            .collect::<Vec<_>>();

        let library_operand_mnemonics = vec![
            Mnemonic::SQRT,
            Mnemonic::LN,
            Mnemonic::EXP,
            Mnemonic::READ,
            Mnemonic::PRINT,
            Mnemonic::SIN,
            Mnemonic::COS,
            Mnemonic::TAN,
            Mnemonic::ATN,
            Mnemonic::STOP,
            Mnemonic::LINE,
            Mnemonic::INT,
            Mnemonic::FRAC,
            Mnemonic::FLOAT,
            Mnemonic::CAPTN,
            Mnemonic::PAGE,
            Mnemonic::RND,
            Mnemonic::ABS,
        ];
        let mut library_operand_pwords = library_operand_mnemonics
            .into_iter()
            .map(|m| PWord::new(m, None.into(), StoreOperand::None))
            .collect::<Vec<_>>();

        let mut pwords = vec![];
        pwords.append(&mut store_operand_pwords);
        pwords.append(&mut library_operand_pwords);

        let expected = from_pwords(&pwords);
        assert_eq!(actual.len(), expected.len() + 1);
        let results = actual.iter().skip(1).zip(expected.iter());
        for (line, expected) in results {
            // Line by line assert easier to detect errors.
            assert_eq!(line, expected)
        }
    }

    //     #[test]
    //     fn invalid_syntax() {
    //         let program = r#"
    // 0001
    // 0002    Invalid One
    // 0003
    // 0004    SQRT        ; Valid
    // 0005
    // 0006    Invalid Two
    // "#;
    //         let actual = parse(program);
    //         let expected = vec![
    //             Err(Error::FailedToParse("0001".into())),
    //             Err(Error::FailedToParse("0002    Invalid One".into())),
    //             Err(Error::FailedToParse("0003".into())),
    //             Ok(SourceProgramLine::new(4, SourceProgramWord::PWord(PWord::new(Mnemonic::SQRT)), "; Valid".into())),
    //             Err(Error::FailedToParse("0005".into())),
    //             Err(Error::FailedToParse("0006    Invalid Two".into())),
    //         ];
    //         assert_eq!(actual[1..], expected)
    //     }
}
