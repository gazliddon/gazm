use std::fs::File;
use std::io::Read;

#[allow(dead_code)]
pub fn load_file(file_name: &str) -> Vec<u8> {
    let mut file = File::open(file_name).unwrap();
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data).unwrap();
    data
}

#[allow(dead_code)]
pub fn load_file_as_string(file_name: &str) -> String {
    let mut file = File::open(file_name).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents
}
