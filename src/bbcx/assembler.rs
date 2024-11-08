use super::assembly::{Assembly, Code};
use super::ast::SourceProgramLine;

use crate::result::{Error, Result};

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Assembler {}

impl Assembler {
    pub fn assemble(ast: &[SourceProgramLine]) -> Result<Assembly> {
        validate_ast(ast)?;
        let code = generate_code(ast);
        let assembly = Assembly::new(&code);
        println!("{:?}", assembly);
        Ok(assembly)
    }
}

fn validate_ast(ast: &[SourceProgramLine]) -> Result<()> {
    let mut invalid_locations = ast
        .iter()
        .fold(HashMap::new(), |mut counts, line| {
            *counts.entry(line.location()).or_insert(0) += 1;
            counts
        })
        .into_iter()
        .filter(|&(_key, value)| (value > 1))
        .map(|(key, _value)| key)
        .collect::<Vec<_>>();

    if invalid_locations.is_empty() {
        Ok(())
    } else {
        invalid_locations.sort();
        let invalid_locations = invalid_locations
            .into_iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let error = format!(
            "Same location(s) used multiple times: {}",
            invalid_locations
        );
        Err(Error::FailedToAssemble(error))
    }
}

fn generate_code(ast: &[SourceProgramLine]) -> Code {
    ast.iter()
        .map(|line| (*line.location(), line.source_program_word().clone()))
        .collect::<Code>()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bbcx::{ast::*, Parser};

    type SourceProgram = Vec<SourceProgramLine>;

    fn parse(input: &str) -> SourceProgram {
        input
            .to_string()
            .lines()
            .map(|l| Parser::parse_line(l))
            .filter_map(Result::ok)
            .collect::<SourceProgram>()
    }

    #[test]
    fn requires_unique_locations() {
        let program = r#"
0001    JUMP    0001
0002    JUMP    HERE
"#;
        let program = parse(program);
        let assembly = Assembler::assemble(&program).unwrap();
        println!("rul: {:?}", assembly);
        assert_eq!(assembly.content(0), None);
        assert_eq!(
            assembly.content(1),
            Some(SourceProgramWord::PWord(PWord::new(
                Mnemonic::JUMP,
                None.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::NumericAddress(1)),
                    None
                ))
            )))
        );
        assert_eq!(
            assembly.content(2),
            Some(SourceProgramWord::PWord(PWord::new(
                Mnemonic::JUMP,
                None.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("HERE".into())),
                    None
                ))
            )))
        );
    }

    #[test]
    fn fails_when_location_not_unique() {
        let program = r#"
0001    JUMP    0001
0001    JUMP    HERE
0002    JUMP    0002
0002    JUMP    THERE
"#;
        let program = parse(program);
        let result = Assembler::assemble(&program).err().unwrap();
        assert_eq!(
            result,
            Error::FailedToAssemble("Same location(s) used multiple times: 1, 2".into())
        );
    }

    #[test]
    fn will_create_intermediate_code_representation() {
        let program = r#"
0001    JUMP    ALPHA
0002    JUMP    EPSILON
"#;
        let program = parse(program);
        let assembly = Assembler::assemble(&program).unwrap();
        assert_eq!(assembly.content(0), None);
        assert_eq!(
            assembly.content(1),
            Some(SourceProgramWord::PWord(PWord::new(
                Mnemonic::JUMP,
                None.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("ALPHA".into())),
                    None
                ))
            )))
        );
        assert_eq!(
            assembly.content(2),
            Some(SourceProgramWord::PWord(PWord::new(
                Mnemonic::JUMP,
                None.into(),
                StoreOperand::AddressOperand(AddressOperand::new(
                    SimpleAddressOperand::DirectAddress(Address::Identifier("EPSILON".into())),
                    None
                ))
            )))
        );
        assert_eq!(assembly.content(3), None);
    }
}
