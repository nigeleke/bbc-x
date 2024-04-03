use crate::bbc3::Bbc3;
use crate::bbcx::BbcX;
use crate::model::LanguageModel;
use crate::result::Result;

use std::path::Path;

pub(crate) enum Language {
    Bbc3(Bbc3),
    BbcX(BbcX),
}

impl Language {
    pub(crate) fn assemble(&self, file: &Path) -> Result<()> {
        match self {
            Language::Bbc3(model) => model.assemble(file),
            Language::BbcX(model) => model.assemble(file),
        }
    }

    pub(crate) fn run(&self, file: &Path) -> Result<()> {
        match self {
            Language::Bbc3(model) => model.run(file),
            Language::BbcX(model) => model.run(file),
        }
    }

    pub(crate) fn list(&self, file: &Path) -> Result<()> {
        match self {
            Language::Bbc3(model) => model.list(file),
            Language::BbcX(model) => model.list(file),
        }
    }
    
}