use crate::args::Args;
use crate::assembler::Assembler;
use crate::assembly::Assembly;
use crate::ast::SourceProgram;
use crate::parser::Parser;
use crate::result::{Error, Result};

use std::path::PathBuf;

pub(crate) fn bbc_x(args: &Args) -> Result<()> {
    let mut results = vec![];

    for file in args.files() {
        let result = build(file);
        results.push(result);
    }

    let results = results.into_iter().filter_map(Result::err).collect::<Vec<Error>>();

    if results.is_empty() {
        Ok(())
    } else {
        Err(Error::BuildErrors(results))
    }
}

fn build(file: PathBuf) -> Result<()> {
    let source_program = parse(&file)?;
    let _assembled_program = assemble(&source_program)?;
    Ok(())
}

fn parse(file: &PathBuf) -> Result<SourceProgram> {
    let content = std::fs::read(file).map_err(|e| Error::CannotReadFile(e.to_string()))?;
    let content = std::str::from_utf8(&content).map_err(|e| Error::CannotReadFile(e.to_string()))?;
    Parser::parse(content)
}

fn assemble(source: &SourceProgram) -> Result<Assembly> {
    Assembler::assemble(source)
}

#[cfg(test)]
mod test {

    #[test]
    #[ignore]
    fn program_parsed_ok() {
        
    }

    #[test]
    #[ignore]
    fn program_parsed_error() {
        
    }

    #[test]
    #[ignore]
    fn program_assembled_ok() {
        
    }

    #[test]
    #[ignore]
    fn program_assembled_error() {
        
    }

    #[test]
    #[ignore]
    fn list_file_not_created() {

    }

    #[test]
    #[ignore]
    fn list_file_created() {
        
    }

    #[test]
    #[ignore]
    fn list_file_created_at_specified_path() {
        
    }

    #[test]
    #[ignore]
    fn program_executed() {

    }

    #[test]
    #[ignore]
    fn trace_file_not_created() {

    }

    #[test]
    #[ignore]
    fn trace_file_created() {
        
    }

    #[test]
    #[ignore]
    fn trace_file_created_at_specified_path() {
        
    }

}