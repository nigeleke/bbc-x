// use crate::result::Result;
// use crate::ast::*;

// // DEFINITION OF THE ASSEMBLER
// // The assembler is a program which accepts as data
// // source code in the form of source program lines. Each
// // source program line corresponds to one main word of
// // object code and translation is strictly on a line-by-line
// // basis. Sometimes the translation of a source program
// // line causes the compilation of a subsidiary word of
// // object code and sometimes it causes a value to be assigned
// // to, or a modification of, an index register.

// #[derive(Debug, PartialEq)]
// struct Assembler {}

// impl Assembler {
//     fn parse_line(line: &str) -> Result<Line> {
//         unimplemented!()
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
// 	fn parse_line_without_comment() {
//         let line = "100 JMP 1200";
//         let line = Assembler::parse_line(&line).unwrap();
//         let line = format!("{:?}", line);
//         assert_eq!(line, r#"Line {
//             location: Location(100),
//             program_word: ProgramWord("JUMP".into()),
//             comment: None,
//         }"#)
//     }

//     #[ignore]
//     #[test]
//     fn parse_line_with_comment() {

//     }

//     #[ignore]
//     #[test]
//     fn parse_bad_location() {

//     }

//     #[ignore]
//     #[test]
//     fn parse_s_program_word() {}

//     #[ignore]
//     #[test]
//     fn parse_p_program_word() {}

//     #[ignore]
//     #[test]
//     fn parse_f_program_word() {}

//     #[ignore]
//     #[test]
//     fn parse_i_program_word() {}

//     #[ignore]
//     #[test]
//     fn parse_bad_program_word() {

//     }

// }
