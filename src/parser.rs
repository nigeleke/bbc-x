use crate::ast::*;
use crate::grammar::*;
use crate::result::{Error, Result};

struct Parser;

impl Parser {
    fn parse(program: &str) -> Result<SourceProgram> {
        let program = program.as_bytes();
        let result = match Grammar::bbc_x().parse(program) {
            Ok(source_program) => Ok(source_program),
            Err(e) => Err(Error::InvalidSource(format!("{:?}", e))),
        };
        println!("parse::result: {:?}", result);
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn sword_without_label() {
        let program = r#"   "TEXT"
"#;
        let program = Parser::parse(program).unwrap();
        let expected: SourceProgram = 
            vec![
                SourceProgramLine::new(
                    None,
                    SourceProgramWord::SWord("TEXT".into()),
                    None
                )
            ];
        assert_eq!(program, expected)
    }

    #[test]
    fn sword_with_label() {
        let program = r#"LABEL: "TEXT"
"#;
        let program = Parser::parse(program).unwrap();
        let expected: SourceProgram = 
            vec![
                SourceProgramLine::new(
                    Some("LABEL".into()),
                    SourceProgramWord::SWord("TEXT".into()),
                    None
                )
            ];
        assert_eq!(program, expected)
    }

    #[test]
    fn sword_with_label_and_comment() {
        let program = r#"LABEL: "TEXT"  ; A comment
"#;
        let program = Parser::parse(program).unwrap();
        let expected: SourceProgram = 
            vec![
                SourceProgramLine::new(
                    Some("LABEL".into()),
                    SourceProgramWord::SWord("TEXT".into()),
                    Some(" A comment".into()),
                )
            ];
        assert_eq!(program, expected)
    }

    #[test]
    #[ignore]
    fn pword_without_label() { }

    #[test]
    #[ignore]
    fn pword_with_label() { }

    #[test]
    #[ignore]
    fn pword_with_label_and_comment() { }

    #[test]
    fn fword_without_label() {
        let program = r#"   3.14
"#;
        let program = Parser::parse(program).unwrap();
        let expected: SourceProgram = 
            vec![
                SourceProgramLine::new(
                    None,
                    SourceProgramWord::FWord(3.14),
                    None
                )
            ];
        assert_eq!(program, expected)
    }

    #[test]
    fn fword_with_label() {
        let program = r#"LABEL: 3.14
"#;
        let program = Parser::parse(program).unwrap();
        let expected: SourceProgram = 
            vec![
                SourceProgramLine::new(
                    Some("LABEL".into()),
                    SourceProgramWord::FWord(3.14),
                    None
                )
            ];
        assert_eq!(program, expected)
    }

    #[test]
    fn fword_with_label_and_comment() {
        let program = r#"LABEL: 3.14 ; Comment
"#;
        let program = Parser::parse(program).unwrap();
        let expected: SourceProgram = 
            vec![
                SourceProgramLine::new(
                    Some("LABEL".into()),
                    SourceProgramWord::FWord(3.14),
                    Some(" Comment".into())
                )
            ];
        assert_eq!(program, expected)
    }

    #[test]
    fn iword_without_label() {
        let program = r#"   42
"#;
        let program = Parser::parse(program).unwrap();
        let expected: SourceProgram = 
            vec![
                SourceProgramLine::new(
                    None,
                    SourceProgramWord::IWord(42),
                    None
                )
            ];
        assert_eq!(program, expected)
    }

    #[test]
    fn iword_with_label() {
        let program = r#"LABEL: +42
"#;
        let program = Parser::parse(program).unwrap();
        let expected: SourceProgram = 
            vec![
                SourceProgramLine::new(
                    Some("LABEL".into()),
                    SourceProgramWord::IWord(42),
                    None
                )
            ];
        assert_eq!(program, expected)
    }

    #[test]
    fn iword_with_label_and_comment() {
        let program = r#"LABEL: -42 ; Comment
"#;
        let program = Parser::parse(program).unwrap();
        let expected: SourceProgram = 
            vec![
                SourceProgramLine::new(
                    Some("LABEL".into()),
                    SourceProgramWord::IWord(-42),
                    Some(" Comment".into())
                )
            ];
        assert_eq!(program, expected)
    }

    #[test]
    fn octal() {
        let program = r#"   S11110000
    P33332222
    F55554444
    I77776666
"#;
        let program = Parser::parse(program).unwrap();
        let expected: SourceProgram = 
            vec![
                SourceProgramLine::new(None, SourceProgramWord::Octal(Octal::S(0o11110000)), None),
                SourceProgramLine::new(None, SourceProgramWord::Octal(Octal::P(0o33332222)), None),
                SourceProgramLine::new(None, SourceProgramWord::Octal(Octal::F(0o55554444)), None),
                SourceProgramLine::new(None, SourceProgramWord::Octal(Octal::I(0o77776666)), None),
            ];
        assert_eq!(program, expected)
    }
}