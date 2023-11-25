mod template;

use anyhow::{anyhow, Context, Result};
use convert_case::{ Case, Casing };
use handlebars::Handlebars;

use serde_json::json;

use std::{
    fs,
    path::{Path, PathBuf},
};

use regex::Regex;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "gs")]
#[structopt(version = "0.1.0")]
#[structopt(about = "git status checker")]
#[structopt(author = "gazaxian")]
#[structopt(rename_all = "kebab-case")]
pub struct Opts {
    #[structopt(short, long, parse(from_os_str))]
    out_file: Option<PathBuf>,
    #[structopt(name = "FILE", parse(from_os_str))]
    paths: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct HelpEntry {
    pub id: String,
    pub file: PathBuf,
    pub text: String,
}

fn get_id<P: AsRef<Path>>(p: P) -> Result<String> {
    let re = Regex::new(r"^err_(.*).md$").context("Illegal regex?")?;
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

impl HelpEntry {
    pub fn new<P: AsRef<Path>>(file: P) -> Result<Self> {
        let file = file.as_ref().into();
        let id = get_id(&file)?;
        let text = fs::read_to_string(&file).context("Reading help file")?;
        Ok(HelpEntry { id, file, text })
    }
}

fn generate_rust_code(_help: &[HelpEntry]) -> String {
    let reg = Handlebars::new();

    let enums: Vec<_> = _help.iter().map(|h| h.id.clone()).collect();
    let data: Vec<(String, String)> = _help
        .iter()
        .map(|h| (h.id.to_string(), h.text.to_string()))
        .collect();
    let data_str: Vec<_> = data
        .into_iter()
        .map(|(id, text)| format!("({id}, String::from(r#\"{text}\"#))"))
        .collect();

    reg.render_template(
        template::TEMPLATE,
        &json!({"enums" : enums.join(",\n\t"),"data" : data_str.join(",\n\t\t")}),
    )
    .unwrap()
}

fn main() -> Result<()> {
    let opts = Opts::from_args();

    let all: Result<Vec<HelpEntry>> = opts.paths.iter().map(HelpEntry::new).collect();

    let all = all.context("Loading help files")?;

    if let Some(out_file) = opts.out_file {
        let text = generate_rust_code(&all);
        println!("{text}");
        println!("Now write {out_file:?}");
    }

    Ok(())
}
