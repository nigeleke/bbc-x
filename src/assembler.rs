use crate::assembly::*;
use crate::ast::*;
use crate::result::{Error, Result};

use std::collections::HashMap;

// DEFINITION OF THE ASSEMBLER
// The assembler is a program which accepts as data
// source code in the form of source program lines. Each
// source program line corresponds to one main word of
// object code and translation is strictly on a line-by-line
// basis. Sometimes the translation of a source program
// line causes the compilation of a subsidiary word of
// object code and sometimes it causes a value to be assigned
// to, or a modification of, an index register.

#[derive(Debug, PartialEq)]
pub(crate) struct Assembler {}

impl Assembler {
    pub(crate) fn assemble(program: &SourceProgram) -> Result<Assembly> {
        let symbol_table = get_definitions(program)?;
        let references = get_references(program);
        confirm_all_references_defined(&references, &symbol_table)?;

        let code = generate_code(program);

        let assembly = Assembly::new(&symbol_table, &code);
        Ok(assembly)
    }

}

fn get_definitions(program: &SourceProgram) -> Result<SymbolTable> {
    let mut definitions = vec![];
    let mut address = 0;

    for line in program {
        if let Some(identifier) = line.location() {
            definitions.push((identifier.clone(), address));
        }
        if line.source_program_word().is_some() {
            address += 1;
        }
    }

    let mut counted: HashMap<Identifier, usize> = HashMap::new();

    for (id, _) in definitions.clone() {
        let entry = counted.entry(id).or_insert(0);
        *entry += 1;
    }

    let counted = counted
        .into_iter()
        .filter(|(_, c)| *c > 1)
        .map(|(i, _)| i)
        .collect::<Vec<Identifier>>();

    if counted.is_empty() {
        Ok(definitions.into_iter().collect())
    } else {
        Err(Error::DuplicatedSymbols(counted))
    }
}

fn get_references(program: &SourceProgram) -> Vec<Identifier> {
    let mut references = vec![];

    for line in program {
        if let Some(SourceProgramWord::PWord(pword)) = line.source_program_word() {
            if let Some(id) = pword.identifier() {
                references.push(id);
            }
        }
    }

    references
}

fn confirm_all_references_defined(references: &[Identifier], symbol_table: &SymbolTable) -> Result<()> {
    let undefined = references
        .iter()
        .filter(|&k| !symbol_table.contains_key(k))
        .cloned()
        .collect::<Vec<String>>();
    
    if undefined.is_empty() {
        Ok(())
    } else {
        Err(Error::UndefinedSymbols(undefined))
    }
}

fn generate_code(program: &SourceProgram) -> Code {
    program
        .iter()
        .filter(|l| l.source_program_word().is_some())
        .map(|l| l.source_program_word().as_ref().unwrap().clone())
        .collect::<Code>()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn will_create_symbol_table_of_identifiers() {
        let program = r#"
ALPHA:                              ; Comment 1
BETA:   JUMP    ALPHA
GAMMA:                              ; Comment 2
DELTA:  JUMP    EPSILON
EPSILON:      
"#;
        let program = Parser::parse(program).unwrap();
        let assembly = Assembler::assemble(&program).unwrap();
        assert_eq!(assembly.symbol("ALPHA"), Some(0));
        assert_eq!(assembly.symbol("BETA"), Some(0));
        assert_eq!(assembly.symbol("GAMMA"), Some(1));
        assert_eq!(assembly.symbol("DELTA"), Some(1));
        assert_eq!(assembly.symbol("EPSILON"), Some(2));
        assert_eq!(assembly.symbol("ZETA"), None);
    }

    #[test]
    fn will_fail_when_identifier_reference_is_undefined() {
        let program = r#"
ALPHA:                              ; Comment 1
BETA:   JUMP    DELTA
GAMMA:                              ; Comment 2        
"#;
        let program = Parser::parse(program).unwrap();
        let result = Assembler::assemble(&program).err().unwrap();
        assert_eq!(result, Error::UndefinedSymbols(vec!["DELTA".into()]));
    }

    #[test]
    fn will_fail_when_identifier_defined_multiple_times() {
        let program = r#"
ALPHA:                              ; Comment 1
BETA:   JUMP    DELTA
ALPHA:                              ; Comment 2        
"#;
        let program = Parser::parse(program).unwrap();
        let result = Assembler::assemble(&program).err().unwrap();
        assert_eq!(result, Error::DuplicatedSymbols(vec!["ALPHA".into()]));
    }

    #[test]
    fn will_create_intermediate_code_representation() {
        let program = r#"
ALPHA:                              ; Comment 1
BETA:   JUMP    ALPHA
GAMMA:                              ; Comment 2
DELTA:  JUMP    EPSILON
EPSILON:      
"#;
        let program = Parser::parse(program).unwrap();
        let assembly = Assembler::assemble(&program).unwrap();
        assert_eq!(assembly.content(0), Some(SourceProgramWord::PWord(PWord::PutType(Mnemonic::JUMP, None, AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("ALPHA".into())), None)))));
        assert_eq!(assembly.content(1), Some(SourceProgramWord::PWord(PWord::PutType(Mnemonic::JUMP, None, AddressOperand::new(SimpleAddressOperand::DirectAddress(Address::Identifier("EPSILON".into())), None)))));
        assert_eq!(assembly.content(2), None);
    }

}
