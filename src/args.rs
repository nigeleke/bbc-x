use clap::{Parser as ClapParser, ValueEnum};
#[cfg(test)]
use clap::error::Error as ClapError;

use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub(crate) enum Language {
    Bbc3,
    BbcX
}

#[derive(Clone, Debug, ClapParser, PartialEq)]
#[command(version, about, long_about = None)]
///
/// Resurrection of the educational BBC-X assembler language used at Hatfield Polytechnic.
///
pub(crate) struct Args {
    /// Specify the source file language. It is expected that all source files are in the same
    /// language.
    #[arg(long, visible_alias="lang", value_enum, default_value_t=Language::BbcX)]
    language: Language,

    /// Create listing files during compilation. The list files will be named '<FILE>.lst'.
    /// See also [list-path].
    #[arg(short, long, required(false))]
    list: bool,

    /// The folder where the list files will be written. If not specified then they will be
    /// written to the same folder as the input file. Implies '--list'.
    #[arg(long)]
    list_path: Option<PathBuf>,

    /// Run the file(s) following successfully compillation. If more than one file is provided
    /// then each will be run sequentially.
    #[arg(short, long, required(false))]
    run: bool,

    /// Trace a file when it is executed. The trace files will be named '<FILE>.out'
    /// See also [trace-path]. Implies '--run'.
    #[arg(short, long, required(false))]
    trace: bool,

    /// The folder where the trace output files will be written. If not specified then they will
    /// be written to same folder as the input file. Implies '--trace'.
    #[arg(long)]
    trace_path: Option<PathBuf>,

    /// The source file(s) to be compiled and / or run.
    #[arg(required(true))]
    files: Vec<PathBuf>,
}

impl Args {
    pub(crate) fn from(args: Vec<String>) -> Self {
        Args::parse_from(args)
    }

    pub(crate) fn language(&self) -> Language {
        self.language
    }

    #[inline]
    pub(crate) fn list(&self) -> bool {
        self.list
    }

    #[inline]
    pub(crate) fn list_path(&self) -> Option<PathBuf> {
        self.list_path.clone()
    }

    #[inline]
    pub(crate) fn files(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.files.iter().cloned()
    }

    #[inline]
    pub(crate) fn run(&self) -> bool {
        self.run
    }

    #[cfg(test)]
    pub(crate) fn try_from(args: &str) -> Result<Self, ClapError> {
        let args = Vec::from_iter(args.split(' ').map(String::from));
        Args::try_parse_from(&args)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn error_with_zero_args() {
        let args = "bbc-x";
        let result = Args::try_from(&args).map_err(|e| e.kind());
        assert_eq!(result, Err(ErrorKind::MissingRequiredArgument))
    }

    #[test]
    fn use_source_files_1() {
        let args = "bbc-x infile1.bbc";
        let result = Args::try_from(&args).expect("Expected successful parse");
        let files = result.files;
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], PathBuf::new().join("infile1.bbc"));
        assert!(!result.list);
        assert_eq!(result.list_path, None);
        assert!(!result.run);
        assert!(!result.trace);
        assert_eq!(result.trace_path, None);
    }

    #[test]
    fn use_source_files_n() {
        let args = "bbc-x infile1.bbc infile2.bbc infile3.bbc";
        let result = Args::try_from(&args).expect("Expected successful parse");
        let files = result.files;
        assert_eq!(files.len(), 3);
        assert_eq!(files[0], PathBuf::new().join("infile1.bbc"));
        assert_eq!(files[1], PathBuf::new().join("infile2.bbc"));
        assert_eq!(files[2], PathBuf::new().join("infile3.bbc"));
        assert!(!result.list);
        assert_eq!(result.list_path, None);
        assert!(!result.run);
        assert!(!result.trace);
        assert_eq!(result.trace_path, None);
    }

    #[test]
    fn use_list() {
        let args = "bbc-x --list infile1.bbc";
        let result = Args::try_from(&args).expect("Expected successful parse");
        assert!(result.list)
    }

    #[test]
    fn use_list_path() {
        let args = "bbc-x --list-path my/list/path/ infile1.bbc";
        let result = Args::try_from(&args).expect("Expected successful parse");
        assert_eq!(result.list_path, Some(PathBuf::new().join("my/list/path/")))
    }

    #[test]
    fn use_run() {
        let args = "bbc-x --run infile1.bbc";
        let result = Args::try_from(&args).expect("Expected successful parse");
        assert!(result.run)
    }

    #[test]
    fn use_trace() {
        let args = "bbc-x --trace infile1.bbc";
        let result = Args::try_from(&args).expect("Expected successful parse");
        assert!(result.trace)
    }

    #[test]
    fn use_trace_path() {
        let args = "bbc-x --trace-path my/trace/path/ infile1.bbc";
        let result = Args::try_from(&args).expect("Expected successful parse");
        assert_eq!(result.trace_path, Some(PathBuf::new().join("my/trace/path/")))
    }

}
