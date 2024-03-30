mod assembler;
mod assembly;
mod ast;
mod grammar;
mod parser;

use self::assembler::Assembler;
use self::assembly::Assembly;
use self::ast::SourceProgramLine;
use self::parser::Parser;

use crate::list_writer::ListWriter;
use crate::model::*;

#[derive(Clone)]
pub(crate) struct Bbc3 {}

impl Bbc3 {
    pub(crate) fn new() -> Self {
        Self { }
    }
}

impl LanguageModel for Bbc3 {
    type ParsedLine = SourceProgramLine;
    type AbstractSyntaxTree = Vec<SourceProgramLine>;
    type IntermediateCode = Assembly;

    fn parse_line(&self, input: &str) -> ParserResult<Self::ParsedLine> {
        Parser::parse_line(input)
    }
    
    fn ast_from_parsed_lines(&self, lines: &Vec<Self::ParsedLine>) -> Self::AbstractSyntaxTree {
        lines.clone()
    }

    fn assemble(&self, ast: &Self::AbstractSyntaxTree) -> AssemblerResult<Self::IntermediateCode> {
        Assembler::assemble(&ast)
    }
    
    fn run(&self, _ic: &Self::IntermediateCode) -> RuntimeResult<()> {
        println!("BBC-3 run command is not implemented");
        Ok(())
    }

    fn list_line(&self, writer: &mut ListWriter, line: &ParserResult<Self::ParsedLine>) {
        let line = match line {
            Ok(line) => format!("        {}", line.to_string()),
            Err(ParserError::FailedToParseLine(error)) => format!(" *****  {}", error), 
            _ => unreachable!(),
        };
        writer.add_lines_to_listing(&line);
    }
}
