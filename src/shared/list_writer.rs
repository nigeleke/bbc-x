// use crate::args::Args;
// use crate::assembly::{Assembly, SymbolTable};
// use crate::bbc3::ast::SourceProgram;
// use crate::result::{Error, Result, SymbolError};

// use std::path::{Path, PathBuf};

// #[derive(Default)]
// pub(crate) struct ListWriter {
//     list_file: Option<PathBuf>,
//     parsed_result: Option<Result<SourceProgram>>,
//     assembled_result: Option<Result<Assembly>>,
// }

// impl ListWriter {

//     pub(crate) fn new(file: &Path, args: &Args) -> Self {
//         let list_file = get_list_file(file, args);
//         Self { list_file, ..ListWriter::default() }
//     }

//     #[inline]
//     pub(crate) fn with_parsed_result(&mut self, result: Result<SourceProgram>) {
//         self.parsed_result = Some(result);
//     }

//     #[inline]
//     pub(crate) fn with_assembled_result(&mut self, result: Result<Assembly>) {
//         self.assembled_result = Some(result);
//     }

//     pub(crate) fn create_list_file(&self) -> Result<()> {
//         self.list_file.as_ref().map(|file| {
//             let mut listing = Vec::new();

//             add_title_to_listing(&mut listing, &file);

//             let Some(ref parsed_result) = self.parsed_result
//                 else { panic!("ListWriter::create_list_file: Expected parsed result"); };
        
//             match parsed_result {
//                 Ok(source) => {
//                     let Some(ref assembled_result) = self.assembled_result
//                         else { panic!("ListWriter::create_list_file: Expected assembled result"); };

//                     match assembled_result {
//                         Ok(assembly) => add_source_and_symbol_table_to_listing(&mut listing, source, assembly),
//                         Err(e) => add_assembled_errors_to_listing(&mut listing, source, e),
//                     }
//                 },
//                 Err(e) => add_parsed_errors_to_listing(&mut listing, e),
//             };

//             write_content_to_file(file, &listing)
//         }).unwrap_or(Ok(()))
//     }

// }

// fn get_list_file(file: &Path, args: &Args) -> Option<PathBuf> {
//     if args.list() {
//         let parent = file.parent().unwrap().to_path_buf();
//         let parent = args.list_path().unwrap_or(parent);
//         let stem = file.file_stem().unwrap();
//         let list_filename = parent.join(stem).with_extension("lst");
//         Some(list_filename)
//     } else {
//         None
//     }
// }

// fn add_title_to_listing(listing: &mut Vec<String>, file: &Path) {
//     let filename = file.display().to_string();
//     let format = time::format_description::parse(
//         "[weekday repr:short] [day] [month repr:short] [year] [hour]:[minute]",)
//         .unwrap();
//     let now = time::OffsetDateTime::now_utc()
//         .format(&format)
//         .unwrap()
//         .to_string();
//     listing.push(format!("{:<14}{:<42} {}\n", "", filename, now).to_uppercase());
// }

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

// fn add_text_to_listing(listing: &mut Vec<String>, text: &str) {
//     let text = text.split('\n');
//     text.into_iter().for_each(|l| listing.push(l.trim_end().into()));
// }

// #[inline]
// fn write_content_to_file(file: &PathBuf, listing: &[String]) -> Result<()> {
//     std::fs::write(file, listing.join("\n")).map_err(|e| Error::UnableToWriteFile(e.to_string()))
// }
