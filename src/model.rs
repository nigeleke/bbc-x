/// The language model enables concrete implementations of the model to parse, assemble 
/// and execute source code.
/// 
/// The model assumes a language that can be sensibly parsed on a line by line basis,
/// which is the case for BBC-3 & BBC-X.
/// 
/// Concrete implementations need to provide:
///   1. a single line parser to create a parsed line from the source.
///   2. an assembler to convert multiple successfully parsed line into an intermediate code representation.
///   3. an executor to run the intermediate code.
/// 
use crate::result::*;
use crate::list_writer::ListWriter;

use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub(crate) enum ParserError {
    FailedToParseLine(String),
    FailedToParse(String),
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::FailedToParseLine(s) => write!(f, "{}", s),
            ParserError::FailedToParse(s) => write!(f, "{}", s),
        }
    }
}

pub(crate) type ParserResult<T> = std::result::Result<T, ParserError>;

#[derive(Debug, PartialEq)]
pub(crate) enum AssemblerError {
    FailedToAssemble(String),
}

impl std::fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblerError::FailedToAssemble(s) => write!(f, "{}", s),
        }
    }
}

pub(crate) type AssemblerResult<T> = std::result::Result<T, AssemblerError>;

pub(crate) enum RuntimeError {
    FailedToExecute(String)
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::FailedToExecute(s) => write!(f, "{}", s),
        }
    }
}

pub(crate) type RuntimeResult<T> = std::result::Result<T, RuntimeError>;

pub(crate) trait LanguageModel {
    type ParsedLine;
    type AbstractSyntaxTree;
    type IntermediateCode;

    fn parse_line(&self, input: &str) -> ParserResult<Self::ParsedLine>;
    fn ast_from_parsed_lines(&self, lines: &Vec<Self::ParsedLine>) -> Self::AbstractSyntaxTree;
    fn assemble(&self, ast: &Self::AbstractSyntaxTree) -> AssemblerResult<Self::IntermediateCode>;
    fn run(&self, ic: &Self::IntermediateCode) -> RuntimeResult<()>;
    fn list_line(&self, writer: &mut ListWriter, line: &ParserResult<Self::ParsedLine>);

    fn parse(&self, input: &str) -> ParserResult<Self::AbstractSyntaxTree> {
        let lines = input.lines();
        let lines_count = lines.clone().count();
        
        let all_results = lines.clone().map(|l| self.parse_line(l));
        let ok_results = Vec::from_iter(all_results.clone()
            .filter_map(|l| l.ok()));

        let all_ok = lines_count == ok_results.len();

        if all_ok {
            Ok(self.ast_from_parsed_lines(&ok_results))
        } else {
            let all_results = all_results
                .zip(lines)
                .map(|(r, l)|
                    match (r, l) {
                        (Ok(_), l) => format!("        {}", l), 
                        (Err(ParserError::FailedToParseLine(e)), l) => format!(" *****  {}\n         {}", l, e),
                        _ => unreachable!(),
                    })
                .collect::<Vec<_>>()
                .join("\n");
            Err(ParserError::FailedToParse(all_results))
        }
    }

    fn build(&self, file: &PathBuf) -> Result<()> {
        let filename = file.display().to_string();

        let content = self.get_file_content(file)?;

        let ast = self.parse(&content)
            .map_err(|e| Error::FailedToParseFile(filename.clone(), e.to_string()))?;

        let ic = self.assemble(&ast)
            .map_err(|e| Error::FailedToAssembleFile(filename.clone(), e.to_string()))?;

        _ = self.run(&ic)
            .map_err(|e| Error::FailedToRunFile(filename.clone(), e.to_string()))?;

        Ok(())
    }

    fn get_file_content(&self, file: &PathBuf) -> Result<String> {
        let filename = file.display().to_string();

        let content = std::fs::read(file)
            .map_err(|e| Error::CannotReadFile(filename.clone(), e.to_string()))?;

        String::from_utf8(content)
            .map_err(|e| Error::CannotReadFile(filename.clone(), e.to_string()))
    }

    fn list(&self, file: &PathBuf, writer: &mut ListWriter) -> Result<()> {
        let filename = file.display().to_string();
        let content = self.get_file_content(file)?;

        content
            .lines()
            .map(|l| self.parse_line(l))
            .for_each(|r| self.list_line(writer, &r));

        writer.write_content_to_file().map_err(|e| Error::CannotToWriteFile(filename, e.to_string()))
    }

}
