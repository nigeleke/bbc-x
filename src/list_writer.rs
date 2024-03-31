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
