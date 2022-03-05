

pub trait PathSearcher {
    fn get_full_path(&self, file : &str) -> Result<PathBuf>;
}


pub struct Paths {
    paths : Vec<PathBuf>
}
