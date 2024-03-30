use crate::args::Args;

use std::path::{Path, PathBuf};

pub(crate) struct ListWriter {
    list_file: Option<PathBuf>,
    listing: Vec<String>,
}

impl ListWriter {

    pub(crate) fn new(file: &Path, args: &Args) -> Self {
        let list_file = ListWriter::get_list_file(file, args);
        let listing = Vec::new();
        let mut me = Self { list_file, listing };
        me.add_title_to_listing(file);
        me
    }

    fn get_list_file(file: &Path, args: &Args) -> Option<PathBuf> {
        if args.list() {
            let parent = file.parent().unwrap().to_path_buf();
            let parent = args.list_path().unwrap_or(parent);
            let stem = file.file_stem().unwrap();
            let list_filename = parent.join(stem).with_extension("lst");
            Some(list_filename)
        } else {
            None
        }
    }

    fn add_title_to_listing(&mut self, file: &Path) {
        let filename = file.display().to_string();
        let format = time::format_description::parse(
            "[weekday repr:short] [day] [month repr:short] [year] [hour]:[minute]",)
            .unwrap();
        let now = time::OffsetDateTime::now_utc()
            .format(&format)
            .unwrap()
            .to_string();
        self.add_line_to_listing(&format!("{:<14}{:<42} {}\n", "", filename, now).to_uppercase());
    }

    pub(crate) fn add_lines_to_listing(&mut self, text: &str) {
        let text = text.split('\n');
        text.into_iter().for_each(|s| self.add_line_to_listing(s));
    }
    
    fn add_line_to_listing(&mut self, text: &str) {
        let line_number = self.listing.len() + 1;
        let text = format!("{:>5} {}", line_number, text);
        self.listing.push(text.trim_end().into())
    }

    #[inline]
    pub(crate) fn write_content_to_file(&self) -> std::result::Result<(), std::io::Error> {
        if let Some(list_file) = &self.list_file {
            std::fs::write(list_file, self.listing.join("\n"))
        } else {
            Ok(())
        }
    }
    
}



// fn add_parsed_errors_to_listing(listing: &mut Vec<String>, error: &Error) {
//     let Error::InvalidInput(parsed_lines) = error else {
//         panic!("list_writer::add_parsed_errors_to_listing: Unexpected error {:?}", error);
//     };

//     for parsed_line in parsed_lines {
//         let text = match parsed_line.parse_result() {
//             Ok(line) => format!("{:>5}{:^9}{}", parsed_line.line_number(), "", line),
//             Err(e) => {
//                 let Error::InvalidLine(error, line) = e else {
//                     panic!("list_writer::add_parsed_errors_to_listing: Unexpected error {:?}", error);
//                 };
//                 format!("{:>5}{:^9}{}\n{}", parsed_line.line_number(), "*****", error, line)
//             },
//         };
//         add_text_to_listing(listing, &text);
//     }
// }

// fn add_assembled_errors_to_listing(listing: &mut Vec<String>, source: &SourceProgram, error: &Error) {
//     add_source_to_listing(listing, source);
//     add_text_to_listing(listing, "\n***** Errors: *****\n");
    
//     let Error::UnresolvedSymbols(errors) = error else {
//         panic!("list_writer::add_assembled_errors_to_listing: Unexpected error {:?}", error);
//     };

//     for error in errors {
//         match error {
//             SymbolError::Duplicated(s) => add_text_to_listing(listing, &format!("Duplicated symbol: {}", s)),
//             SymbolError::Undefined(s) => add_text_to_listing(listing, &format!("Undefined symbol: {}", s)),
//         }
//     }
// }

// fn add_source_and_symbol_table_to_listing(listing: &mut Vec<String>, source: &SourceProgram, assembly: &Assembly) {
//     add_source_to_listing(listing, source);
//     add_symbol_table_to_listing(listing, assembly.symbol_table());
// }

// fn add_source_to_listing(listing: &mut Vec<String>, source: &SourceProgram) {
//     let source = source.iter().enumerate();
//     for (index, line) in source {
//         add_text_to_listing(listing, &format!("{:>5}{:<9}{}", index+1, "", line));
//     }
// }

// fn add_symbol_table_to_listing(listing: &mut Vec<String>, symbol_table: &SymbolTable) {
//     add_text_to_listing(listing, "\nSYMBOL TABLE:\n=============\n");
//     let mut keys = symbol_table.keys().collect::<Vec<_>>();
//     keys.sort();
//     for k in keys {
//         add_text_to_listing(listing, &format!("{:<8}{:08o}", k, symbol_table.get(k).unwrap()));
//     }
// }

