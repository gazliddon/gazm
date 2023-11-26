use std::path::{ PathBuf,Path };
use std::fs;
use regex::Regex;
use anyhow::{ Result,Context, anyhow };
use convert_case::{ Case, Casing };

#[derive(Debug)]
pub struct HelpEntry {
    pub id: String,
    pub file: PathBuf,
    pub text: String,
}

impl HelpEntry {
    pub fn new<P: AsRef<Path>>(file: P) -> Result<Self> {
        let file = file.as_ref().into();
        let id = get_id(&file)?;
        let text = fs::read_to_string(&file).context("Reading help file")?;
        Ok(HelpEntry { id, file, text })
    }
}

fn get_id<P: AsRef<Path>>(p: P) -> Result<String> {
    let re = Regex::new(r"^(.*).md$").context("Illegal regex?")?;
    let file_name_no_path = file_name_no_path(&p);
    re.captures(&file_name_no_path)
        .and_then(|c| c.get(1))
        .map(|c| c.as_str().to_case(Case::Pascal))
        .context(anyhow!("Whoops"))
}

pub fn file_name_no_path<P: AsRef<Path>>(p: P) -> String {
    let p = p.as_ref().to_path_buf();
    let file_name_no_path = p.iter().last().unwrap().to_string_lossy();
    file_name_no_path.to_string()
}

