use crate::error;
use std::collections::HashSet;
use std::fs;

#[derive(Debug)]
pub struct FileNames {
    names: HashSet<String>,
}

impl FileNames {
    pub fn from_path(path: std::path::PathBuf) -> Result<Self, error::LinkError> {
        let files = fs::read_dir(path)?;
        let mut names = HashSet::new();
        for file in files {
            let file_path = file?.path();
            let file_stem = file_path.file_stem();
            if let Some(name) = file_stem {
                let name = name.to_owned().into_string();
                if let Ok(value) = name {
                    names.insert(value);
                }
            }
        }
        Ok(FileNames { names })
    }
}
