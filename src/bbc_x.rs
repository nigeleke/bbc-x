use crate::args::Args;
use crate::assembler::Assembler;
use crate::assembly::Assembly;
use crate::ast::SourceProgram;
use crate::list_writer::ListWriter;
use crate::parser::Parser;
use crate::result::{Error, Result};

use std::path::PathBuf;

pub(crate) fn bbc_x(args: &Args) -> Result<()> {
    let mut results = vec![];

    for file in args.files() {
        let result = build(&file, args);
        results.push(result);
    }

    let results = results.into_iter().filter_map(Result::err).collect::<Vec<Error>>();

    if results.is_empty() {
        Ok(())
    } else {
        Err(Error::BuildErrors(results))
    }
}

fn build(file: &PathBuf, args: &Args) -> Result<()> {
    let mut list_writer = ListWriter::new(file, args);

    let result = parse(file);
    list_writer.with_parsed_result(result.clone());

    match result {
        Ok(source) => {
            let result = assemble(&source);
            list_writer.with_assembled_result(result.clone());
            list_writer.create_list_file()?;
            result.map(|_| ())
        },
        Err(_) => {
            list_writer.create_list_file()?;
            result.map(|_| ())
        },
    }
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
    use super::*;

    use tempdir::TempDir;

    #[test]
    fn program_parsed_ok() {
        let args = vec!["bbc-x", "./examples/test/nthg.bbc"].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let result = bbc_x(&args);
        assert!(result.is_ok())        
    }

    #[test]
    fn program_parsed_error() {
        let args = vec!["bbc-x", "./examples/test/invalid_syntax.bbc"].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let result = bbc_x(&args);
        assert!(result.is_err())        
    }

    #[test]
    fn program_assembled_ok() {
        let args = vec!["bbc-x", "./examples/test/nthg.bbc"].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let result = bbc_x(&args);
        assert!(result.is_ok())                
    }

    #[test]
    fn program_assembled_error() {
        let args = vec!["bbc-x", "./examples/test/invalid_semantics.bbc"].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let result = bbc_x(&args);
        assert!(result.is_err())                
    }

    #[test]
    fn list_file_not_created() {
        let temp_folder = TempDir::new("bbcx-tests").unwrap();

        let temp_target = temp_folder.path().join("nthg.bbc");
        let temp_target_str = temp_target.display().to_string();

        std::fs::copy("./examples/test/nthg.bbc", temp_target).unwrap();

        let args = vec!["bbc-x", &temp_target_str].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let _ = bbc_x(&args);

        let list_target = temp_folder.path().join("nthg.lst");
        assert!(!list_target.exists());
    }

    #[test]
    fn list_file_created() {
        let temp_folder = TempDir::new("bbcx-tests").unwrap();

        let temp_target = temp_folder.path().join("nthg.bbc");
        let temp_target_str = temp_target.display().to_string();

        std::fs::copy("./examples/test/nthg.bbc", temp_target).unwrap();

        let args = vec!["bbc-x", "--list", &temp_target_str].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let _ = bbc_x(&args);

        let list_target = temp_folder.path().join("nthg.lst");
        assert!(list_target.exists());
    }

    #[test]
    fn list_file_created_when_parsed_error() {
        let temp_folder = TempDir::new("bbcx-tests").unwrap();

        let temp_target = temp_folder.path().join("invalid_syntax.bbc");
        let temp_target_str = temp_target.display().to_string();

        std::fs::copy("./examples/test/invalid_syntax.bbc", temp_target).unwrap();

        let args = vec!["bbc-x", "--list", &temp_target_str].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let _ = bbc_x(&args);

        let list_target = temp_folder.path().join("invalid_syntax.lst");
        assert!(list_target.exists());
    }

    #[test]
    fn list_file_created_when_assembled_error() {
        let temp_folder = TempDir::new("bbcx-tests").unwrap();

        let temp_target = temp_folder.path().join("invalid_semantics.bbc");
        let temp_target_str = temp_target.display().to_string();

        std::fs::copy("./examples/test/invalid_semantics.bbc", temp_target).unwrap();

        let args = vec!["bbc-x", "--list", &temp_target_str].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let _ = bbc_x(&args);

        let list_target = temp_folder.path().join("invalid_semantics.lst");
        assert!(list_target.exists());
    }

    #[test]
    fn list_file_created_at_specified_path() {
        let temp_folder = TempDir::new("bbcx-tests").unwrap();

        let temp_target = temp_folder.path();
        let temp_target_str = temp_target.display().to_string();

        let args = vec!["bbc-x", "--list", "--list-path", &temp_target_str, "./examples/test/nthg.bbc"].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let _ = bbc_x(&args);

        let list_target = temp_folder.path().join("nthg.lst");
        assert!(list_target.exists());        
    }

    #[test]
    fn list_file_lists_all_operations() {
        let temp_folder = TempDir::new("bbcx-tests").unwrap();

        let temp_target = temp_folder.path().join("instruction_set.bbc");
        let temp_target_str = temp_target.display().to_string();

        std::fs::copy("./examples/test/instruction_set.bbc", temp_target).unwrap();

        let args = vec!["bbc-x", "--list", &temp_target_str].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let _ = bbc_x(&args);

        let list_target = temp_folder.path().join("instruction_set.lst");
        assert!(list_target.exists());
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