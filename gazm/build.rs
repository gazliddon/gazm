use std::io::Result;

pub fn main() -> Result<()> {
    use glob::glob;

    for entry in glob( "assets/help/*.md").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => println!("{:?}", path.display()),
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(())
}
