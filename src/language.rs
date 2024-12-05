use crate::bbc3::Bbc3;
use crate::bbcx::BbcX;
use crate::model::LanguageModel;
use crate::result::Result;

use std::path::Path;

pub enum Language {
    Bbc3(Bbc3),
    BbcX(BbcX),
}

impl Language {
    pub fn assemble(&self, file: &Path) -> Result<()> {
        match self {
            Language::Bbc3(model) => model.assemble(file),
            Language::BbcX(model) => model.assemble(file),
        }
    }

    pub fn run(&self, file: &Path, trace: Option<&Path>) -> Result<()> {
        match self {
            Language::Bbc3(model) => model.run(file, trace),
            Language::BbcX(model) => model.run(file, trace),
        }
    }

    pub fn list(&self, file: &Path) -> Result<()> {
        match self {
            Language::Bbc3(model) => model.list(file),
            Language::BbcX(model) => model.list(file),
        }
    }
}
