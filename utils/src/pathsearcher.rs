use std::path::{Path, PathBuf};

pub enum SearchError {
    FileNotFound(PathBuf, Vec<PathBuf>),
    Placeholder,
}

pub trait PathSearcher {
    fn get_full_path(&self, file: &Path) -> Result<PathBuf, SearchError>;
}

#[derive(Debug)]
pub struct Paths {
    paths: Vec<PathBuf>,
}

impl Paths {
    pub fn new() -> Self {
        Self {
            paths: vec![]
        }
    }
    pub fn from_paths<P: AsRef<Path>>(paths: &[P]) -> Self {
        let paths = paths.iter().map(|p| p.as_ref().into()).collect();
        let mut ret = Self::new();
        ret.paths = paths;
        ret
    }

    pub fn add_path<P: AsRef<Path>>(&mut self, path: P) {
        self.paths.push(path.as_ref().to_path_buf())
    }
}

impl PathSearcher for Vec<&Paths> {
    fn get_full_path(&self, file: &Path) -> Result<PathBuf, SearchError> {
        let mut tried : Vec<Vec<PathBuf>> = vec![];

        for paths in self {
            match paths.get_full_path(file) {
                Ok(path) => return Ok(path),
                Err(SearchError::FileNotFound(_,paths_tried)) => tried.push(paths_tried),
                Err(e) => return Err(e)
            }
        }

        let tried = tried.into_iter().flatten().collect();

        Err(SearchError::FileNotFound(file.to_path_buf(),tried))
    }
}

impl PathSearcher for Paths {
    fn get_full_path(&self, file_name: &Path) -> Result<PathBuf, SearchError> {
        let mut tried: Vec<_> = vec![file_name.to_path_buf()];

        if !file_name.has_root() {
            for i in &self.paths {
                let x = i.join(file_name);
                if x.exists() {
                    return Ok(x);
                } else {
                    tried.push(x.clone());
                }
            }
        } else {
            if file_name.exists() {
                return Ok(file_name.to_path_buf());
            }
        }

        return Err(SearchError::FileNotFound(file_name.to_path_buf(), tried));
    }
}
