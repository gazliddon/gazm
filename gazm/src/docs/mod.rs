use convert_case::{Case, Casing};
use core::hash::Hash;
use std::{collections::HashMap, fs, io::Result, path::PathBuf};

lazy_static::lazy_static! {

    pub static ref HELP : HelpText<String> = {
    let docs = gather_docs().unwrap();

    HelpText::new(docs)

    };
}

#[derive(Debug)]
pub struct HelpText<K>
where
    K: Hash + PartialEq + Eq,
{
    all_help: HashMap<K, String>,
}

impl<K: Hash + PartialEq + Eq> HelpText<K> {
    pub fn new(all_help: HashMap<K, String>) -> Self {
        Self { all_help }
    }

    pub fn get(&self, name: &K) -> Option<&str> {
        self.all_help.get(name).map(|x| x.as_str())
    }
}

impl HelpText<String> {
    pub fn get_str(&self, name: &str) -> Option<&str> {
        self.all_help.get(name).map(|x| x.as_str())
    }
}

pub fn gather_docs() -> Result<HashMap<String, String>> {
    let x = gather_doc_files().unwrap();
    let all_help: HashMap<String, String> = x
        .into_iter()
        .map(|(path, id)| {
            let text = fs::read_to_string(path).expect("Can't read file");
            (id, text)
        })
        .collect();

    Ok(all_help)
}

pub fn gather_doc_files() -> Result<Vec<(PathBuf, String)>> {
    use glob::glob;
    use regex::Regex;
    let re = Regex::new(r"^err_(.*).md$").unwrap();

    let mut ret = vec![];

    for markdown_file in glob("assets/help/*.md").expect("Failed to read glob pattern") {
        let markdown_file = markdown_file.unwrap();
        let file_name_no_path = markdown_file
            .iter()
            .last()
            .unwrap()
            .to_string_lossy();
        let captures = re.captures(&file_name_no_path).unwrap();

        if let Some(captured) = captures.get(1) {
            let id = captured.as_str().to_case(Case::Pascal);
            ret.push((markdown_file, id));
        }
    }

    Ok(ret)
}
