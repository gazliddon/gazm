///! In memory representation of all source file
use super::{AsmSource, Position, SourceFile, SourceErrorType, SourceInfo};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct SourceFiles {
    pub id_to_source_file: HashMap<u64, SourceFile>,
    pub path_to_id: HashMap<PathBuf, u64>,
}

impl SourceFiles {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_next_id(&self) -> u64 {
        let max = self.id_to_source_file.keys().max();
        max.map(|x| x + 1).unwrap_or(0)
    }

    pub fn add_source_file<P: AsRef<Path>>(&mut self, p: P, text: &str) -> u64 {
        let p = p.as_ref().to_path_buf();
        let id = self.get_next_id();
        let source_file = SourceFile::new(&p, text);
        self.id_to_source_file.insert(id, source_file);
        self.path_to_id.insert(p, id);
        id
    }

    pub fn get_source<P: AsRef<Path>>(&self, p: P) -> Result<&SourceFile, SourceErrorType> {
        let p = p.as_ref().to_path_buf();

        let id = self
            .path_to_id
            .get(&p)
            .ok_or_else(|| SourceErrorType::FileNotFound(p.to_string_lossy().into()))?;
        Ok(self.id_to_source_file.get(id).unwrap())
    }

    pub fn get_source_id(&self, id: u64) -> Result<&SourceFile, SourceErrorType> {
        self.id_to_source_file.get(&id).ok_or_else(|| SourceErrorType::IdNotFound(id))
    }


    pub fn get_source_mut<P: AsRef<Path>>(&mut self, p: P) -> Result<&SourceFile, SourceErrorType> {
        let p = p.as_ref().to_path_buf();

        let id = self
            .path_to_id
            .get(&p)
            .ok_or_else(|| SourceErrorType::FileNotFound(p.to_string_lossy().into()))?;
        Ok(self.id_to_source_file.get_mut(id).unwrap())
    }

    pub fn remove_file<P: AsRef<Path>>(&mut self, p: P) -> Result<(), SourceErrorType> {
        let p = p.as_ref().to_path_buf();

        let id = self
            .path_to_id
            .get(&p)
            .ok_or_else(|| SourceErrorType::FileNotFound(p.to_string_lossy().to_string()))?;
        self.remove_id(*id)
    }

    pub fn remove_id(&mut self, id: u64) -> Result<(), SourceErrorType> {
        let sf = self
            .id_to_source_file
            .get(&id)
            .ok_or_else(|| SourceErrorType::IdNotFound(id))?;
        let p = sf.file.clone();
        self.path_to_id.remove(&p);
        self.id_to_source_file.remove(&id);
        Ok(())
    }

    pub fn get_source_info<'a>(&'a self, pos: &Position) -> Result<SourceInfo<'a>, SourceErrorType> {
        if let AsmSource::FileId(file_id) = pos.src {
            let source_file = self.get_source_id(file_id)?;

            let fragment = source_file.get_span(pos);
            let line_str = source_file.get_line_from_position(pos);

            let ret = SourceInfo {
                line_str,
                col: pos.col,
                line: pos.line,
                fragment,
                source_file,
                file: source_file.file.clone(),
                pos: pos.clone(),
            };

            Ok(ret)
        } else {
            Err(SourceErrorType::Misc)
        }
    }
}
