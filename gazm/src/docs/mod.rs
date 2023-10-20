use std::{io::Result, path::PathBuf};

pub fn gather_docs() -> Result<Vec<(PathBuf,String)>> {
    use glob::glob;
    use regex::Regex;
    let re = Regex::new(r"^err_(.*).md$").unwrap();

    let mut ret = vec![];

    for entry in glob("gazm/assets/help/*.md").expect("Failed to read glob pattern") {
        let entry = entry.unwrap();
        let file_name = entry.iter().last().unwrap().to_string_lossy().to_string();
        let captures = re.captures(&file_name).unwrap();

        if let Some(captured) = captures.get(1) {
            ret.push(( entry, captured.as_str().to_string() ));
        } 
    }
    Ok(ret)
}

