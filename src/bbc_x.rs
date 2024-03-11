use crate::args::Args;
use crate::assembler::Assembler;
use crate::assembly::Assembly;
use crate::ast::SourceProgram;
use crate::parser::Parser;
use crate::result::{Error, Result};

use std::path::PathBuf;

pub(crate) fn bbc_x(args: &Args) -> Result<()> { 
    for file in args.files() {
        let source_program = parse(&file)?;
        let assembled_program = assemble(&source_program)?;
    }

    Ok(())
}

fn parse(file: &PathBuf) -> Result<SourceProgram> {
    let content = std::fs::read(file).map_err(|e| Error::CannotReadFile(e.to_string()))?;
    let content = std::str::from_utf8(&content).map_err(|e| Error::CannotReadFile(e.to_string()))?;
    Parser::parse(&content)
}

fn assemble(source: &SourceProgram) -> Result<Assembly> {
    Assembler::assemble(source)
}
