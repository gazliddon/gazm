use anyhow::{anyhow, Context, Result};
use convert_case::{Case, Casing};
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::{default, fs, marker};

#[derive(Debug)]
pub struct HelpEntry {
    pub id: String,
    pub file: PathBuf,
    pub text: String,
    pub short: String,
}

#[derive(Debug, Deserialize)]
pub struct YamlHeader {
    pub short: String,
}

pub fn split_out_yaml(text: &str) -> Option<(&str, &str)> {
    let rx = Regex::new("---\n(.*)\n---\n(.*)").unwrap();
    let caps = rx.captures(text)?;
    let yaml = caps.get(1)?;
    let rest = caps.get(2)?;
    let rest = &text[rest.range().start..];
    return Some((yaml.as_str(), rest));
}

impl HelpEntry {
    pub fn new<P: AsRef<Path>>(file: P) -> Result<Self> {
        let file = file.as_ref().into();
        let id = get_id(&file)?;
        let text = fs::read_to_string(&file).context("Reading help file")?;

        if let Some((yaml, markdown)) = split_out_yaml(&text) {
            if let Ok(yaml_header) = serde_yaml::from_str::<YamlHeader>(&yaml) {
                return Ok(HelpEntry {
                    id,
                    file,
                    text: markdown.to_owned(),
                    short: yaml_header.short,
                });
            }
        }

        Ok(HelpEntry {
            id,
            file,
            text,
            short: Default::default(),
        })
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
