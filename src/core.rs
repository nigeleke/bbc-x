use crate::args::{Args, Language as LanguageArg};
use crate::bbc3::Bbc3;
use crate::bbcx::BbcX;
use crate::language::Language;
use crate::result::{Error, Result};

pub(crate) struct Core { }

impl Core {
    pub(crate) fn build_all(args: &Args) -> Result<()> {
        let language = match args.language() {
            LanguageArg::Bbc3 => Language::Bbc3(Bbc3::new(args)),
            LanguageArg::BbcX => Language::BbcX(BbcX::new(args)),
        };

        let mut results = vec![];
    
        for file in args.files() {
            let result = language
                .assemble(&file)
                .and_then(|_| if args.run(){ language.run(&file) } else { Ok(()) } );
            results.push(result);

            let _= language.list(&file);
        }
    
        let results = results.into_iter().filter_map(Result::err).collect::<Vec<Error>>();
    
        if results.is_empty() {
            Ok(())
        } else {
            Err(Error::BuildErrors(results))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use tempdir::TempDir;

    #[test]
    fn program_parsed_ok() {
        let args = vec!["bbc-x", "--lang=bbc3", "./examples/test/bbc3/nthg.bbc"].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let result = Core::build_all(&args);
        assert!(result.is_ok())        
    }

    #[test]
    fn program_parsed_error() {
        let args = vec!["bbc-x", "--lang=bbc3", "./examples/test/bbc3/invalid_syntax.bbc"].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let result = Core::build_all(&args);
        assert!(result.is_err())        
    }

    #[test]
    fn program_assembled_ok() {
        let args = vec!["bbc-x", "--lang=bbc3", "./examples/test/bbc3/nthg.bbc"].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let result = Core::build_all(&args);
        assert!(result.is_ok())                
    }

    #[test]
    fn program_assembled_error() {
        let args = vec!["bbc-x", "--lang=bbc3", "./examples/test/bbc3/invalid_semantics.bbc"].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let result = Core::build_all(&args);
        assert!(result.is_err())                
    }

    #[test]
    fn list_file_not_created() {
        let temp_folder = TempDir::new("bbcx-tests").unwrap();

        let temp_target = temp_folder.path().join("nthg.bbc");
        let temp_target_str = temp_target.display().to_string();

        std::fs::copy("./examples/test/bbc3/nthg.bbc", temp_target).unwrap();

        let args = vec!["bbc-x", "--lang=bbc3", &temp_target_str].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let _ = Core::build_all(&args);

        let list_target = temp_folder.path().join("nthg.lst");
        assert!(!list_target.exists());
    }

    #[test]
    fn list_file_created() {
        let temp_folder = TempDir::new("bbcx-tests").unwrap();

        let temp_target = temp_folder.path().join("nthg.bbc");
        let temp_target_str = temp_target.display().to_string();

        std::fs::copy("./examples/test/bbc3/nthg.bbc", temp_target).unwrap();

        let args = vec!["bbc-x", "--lang=bbc3", "--list", &temp_target_str].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let _ = Core::build_all(&args);

        let list_target = temp_folder.path().join("nthg.lst");
        assert!(list_target.exists());
    }

    #[test]
    fn list_file_created_when_parsed_error() {
        let temp_folder = TempDir::new("bbcx-tests").unwrap();

        let temp_target = temp_folder.path().join("invalid_syntax.bbc");
        let temp_target_str = temp_target.display().to_string();

        std::fs::copy("./examples/test/bbc3/invalid_syntax.bbc", temp_target).unwrap();

        let args = vec!["bbc-x", "--lang=bbc3", "--list", &temp_target_str].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let _ = Core::build_all(&args);

        let list_target = temp_folder.path().join("invalid_syntax.lst");
        assert!(list_target.exists());
    }

    #[test]
    fn list_file_created_when_assembled_error() {
        let temp_folder = TempDir::new("bbcx-tests").unwrap();

        let temp_target = temp_folder.path().join("invalid_semantics.bbc");
        let temp_target_str = temp_target.display().to_string();

        std::fs::copy("./examples/test/bbc3/invalid_semantics.bbc", temp_target).unwrap();

        let args = vec!["bbc-x", "--lang=bbc3", "--list", &temp_target_str].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let _ = Core::build_all(&args);

        let list_target = temp_folder.path().join("invalid_semantics.lst");
        assert!(list_target.exists());
    }

    #[test]
    fn list_file_created_at_specified_path() {
        let temp_folder = TempDir::new("bbcx-tests").unwrap();

        let temp_target = temp_folder.path();
        let temp_target_str = temp_target.display().to_string();

        let args = vec!["bbc-x", "--lang=bbc3", "--list", "--list-path", &temp_target_str, "./examples/test/bbc3/nthg.bbc"].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let _ = Core::build_all(&args);

        let list_target = temp_folder.path().join("nthg.lst");
        assert!(list_target.exists());        
    }

    #[test]
    fn list_file_lists_all_operations() {
        let temp_folder = TempDir::new("bbcx-tests").unwrap();

        let temp_target = temp_folder.path().join("instruction_set.bbc");
        let temp_target_str = temp_target.display().to_string();

        std::fs::copy("./examples/test/bbc3/instruction_set.bbc", temp_target).unwrap();

        let args = vec!["bbc-x", "--lang=bbc3", "--list", &temp_target_str].into_iter().map(|s| s.to_string()).collect();
        let args = Args::from(args);
        let _ = Core::build_all(&args);

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