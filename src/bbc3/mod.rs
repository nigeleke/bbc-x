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

    fn impl_parse_line(&self, input: &str) -> ParserResult<Self::ParsedLine> {
        Parser::parse_line(input)
    }
    
    fn impl_parsed_lines_to_ast(&self, lines: &[Self::ParsedLine]) -> Self::AbstractSyntaxTree {
        lines.to_vec()
    }

    fn impl_assemble(&self, ast: &Self::AbstractSyntaxTree) -> AssemblerResult<Self::IntermediateCode> {
        Assembler::assemble(ast)
    }
    
    fn impl_run(&self, _code: &Self::IntermediateCode) -> RuntimeResult<()> {
        Err(RuntimeError::FailedToExecute("BBC-3 run command is not implemented".into()))
    }

    fn impl_list_line(&self, writer: &mut ListWriter, line: &ParserResult<Self::ParsedLine>) {
        let line = match line {
            Ok(line) => format!("        {}", line),
            Err(ParserError::FailedToParseLine(error)) => format!(" *****  {}", error), 
            _ => unreachable!(),
        };
        writer.add_lines_to_listing(&line);
    }
}
