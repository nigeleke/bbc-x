mod assembler;
mod assembly;
mod ast;
mod charset;
mod executor;
mod grammar;
mod memory;
mod parser;

use self::assembler::Assembler;
use self::assembly::Assembly;
use self::ast::SourceLine;
use self::executor::Executor;
use self::parser::Parser;

use crate::args::Args;
use crate::list_writer::ListWriter;
use crate::model::*;
use crate::result::{Error, Result};

use std::path::Path;

pub struct BbcX {
    args: Args,
}

impl BbcX {
    pub fn new(args: &Args) -> BbcX {
        let args = args.clone();
        Self { args }
    }

    fn impl_parse(&self, path: &Path) -> Result<Vec<Result<SourceLine>>> {
        let lines = file_lines(path)?;
        let results = lines.iter().map(|line| Parser::parse_line(line));
        Ok(results.collect())
    }

    fn impl_assemble(&self, path: &Path) -> Result<Assembly> {
        let parsed_lines = self.impl_parse(path)?;

        let parsed_lines_len = parsed_lines.len();
        let ast = parsed_lines
            .iter()
            .filter_map(|l| l.as_ref().ok())
            .cloned()
            .collect::<Vec<_>>();

        let all_ok = parsed_lines_len == ast.len();

        if all_ok {
            Assembler::assemble(&ast)
        } else {
            let lines = file_lines(path)?;
            let all_results = parsed_lines
                .iter()
                .zip(lines.iter())
                .map(|(r, l)| match (r, l) {
                    (Ok(_), l) => format!("        {}", l),
                    (Err(Error::FailedToParse(e)), l) => format!(" *****  {}\n         {}", l, e),
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>()
                .join("\n");
            Err(Error::FailedToAssemble(all_results))
        }
    }

    fn impl_run(&self, path: &Path) -> Result<()> {
        let assembly = self.impl_assemble(path)?;
        let executor = Executor::new();
        _ = executor.execute(&assembly)?;
        Ok(())
    }

    fn impl_list(&self, path: &Path) -> Result<()> {
        let mut writer = ListWriter::new(path, &self.args);
        let lines = file_lines(path)?;
        let results = lines.iter().map(|line| Parser::parse_line(line));
        for line in results {
            let line = match line {
                Ok(line) => format!("        {}", line),
                Err(Error::FailedToParse(error)) => format!(" *****  {}", error),
                _ => unreachable!(),
            };
            writer.add_lines_to_listing(&line);
        }
        writer
            .write_content_to_file()
            .map_err(|e| Error::CannotToWriteFile(path.display().to_string(), e.to_string()))
    }
}

fn file_lines(path: &Path) -> Result<Vec<String>> {
    let filename = path.display().to_string();

    let content =
        std::fs::read(path).map_err(|e| Error::CannotReadFile(filename.clone(), e.to_string()))?;

    let content = String::from_utf8(content)
        .map_err(|e| Error::CannotReadFile(filename.clone(), e.to_string()))?;

    Ok(content.lines().map(|line| line.to_owned()).collect())
}

impl LanguageModel for BbcX {
    fn assemble(&self, path: &Path) -> Result<()> {
        _ = self.impl_assemble(path)?;
        Ok(())
    }

    fn run(&self, path: &Path) -> Result<()> {
        self.impl_run(path)
    }

    fn list(&self, path: &Path) -> Result<()> {
        _ = self.impl_list(path);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn will_assemble() {
        let args = vec!["bbc-x", "--lang=bbc-x", "./examples/test/bbcx/nthg.bbc"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let args = Args::from(args);
        let model = BbcX::new(&args);
        let result = model.assemble(&args.files().next().unwrap());
        assert!(result.is_ok())
    }

    #[test]
    fn will_run() {
        let args = vec![
            "bbc-x",
            "--lang=bbc-x",
            "--run",
            "./examples/test/bbcx/stop.bbc",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
        let args = Args::from(args);
        let model = BbcX::new(&args);
        let result = model.run(&args.files().next().unwrap());
        assert!(result.is_ok())
    }

    #[test]
    fn will_list() {
        let temp_folder = TempDir::new("bbcx-tests-bbcx").unwrap();

        let temp_target = temp_folder.path().join("nthg.bbc");
        let temp_target_str = temp_target.display().to_string();

        std::fs::copy("./examples/test/bbcx/nthg.bbc", temp_target).unwrap();

        let args = vec!["bbc-x", "--lang=bbc-x", "--list", &temp_target_str]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let args = Args::from(args);
        let model = BbcX::new(&args);
        let _ = model.list(&args.files().next().unwrap());

        let list_target = temp_folder.path().join("nthg.lst");
        assert!(list_target.exists());
    }
}
