use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use std::fmt;

#[macro_use]
extern crate serde_derive;

fn main() {
    let dbase = Dbase::from_filename("src/emu/cpu/resources/opcodes.json");
    let source = format!("{}", dbase);

    let out_dir = env::var("OUT_DIR").unwrap();

    let dest_path = Path::new(&out_dir).join("isa_macros.rs");
    let mut f = File::create(&dest_path).unwrap();

    f.write(source.as_bytes()).unwrap();
}

use serde::{Deserialize};
use serde::de::Deserializer;

#[derive(Debug, Clone,  Serialize, Deserialize)]
pub enum AddrMode {
    Indexed,
    Direct,
    Extended,
    Relative,
    Inherent,
    Immediate8,
    Immediate16,
}

// Custome deserializers
fn hex_str_to_num<'de, D>(deserializer: D) -> Result<u16, D::Error>
where D: Deserializer<'de> {
    let hex_string = String::deserialize(deserializer)?;
    let z = u16::from_str_radix(&hex_string, 16).expect("Convert from hex str to u16");
    Ok(z)
}

// Custome deserializers
fn fixup_action<'de, D>(deserializer: D) -> Result<String, D::Error>
where D: Deserializer<'de> {
    let action = String::deserialize(deserializer)?;
    Ok(action.to_lowercase().replace("/", "_"))
}


#[serde(deny_unknown_fields)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instruction {
    // pub display : Option<String>,
    pub addr_mode : AddrMode,
#[serde(deserialize_with = "u16::deserialize")]
    pub cycles : u16,
#[serde(deserialize_with = "fixup_action")]
    pub action : String,
#[serde(deserialize_with = "hex_str_to_num")]
    pub opcode : u16,
#[serde(deserialize_with = "u16::deserialize")]
    pub size : u16,
#[serde(default)]
#[serde(deserialize_with = "u16::deserialize")]
    pub operand_size : u16,
}


impl Instruction {
    pub fn as_macro(&self) -> String {
        format!(
            "0x{:04x} => handle_op!({:?}, {}, 0x{:04x}, {}, {}),",
            self.opcode,
            self.addr_mode,
            self.action,
            self.opcode,
            self.cycles,
            self.size
            )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dbase {
    unknown: Instruction,
    instructions: Vec<Instruction>,
#[serde(skip)]
    lookup: Vec<Instruction>
}

impl Dbase {
    pub fn from_filename(file_name : &str) -> Self {
        let json_str = std::fs::read_to_string(file_name).unwrap();
        let loaded : Dbase = serde_json::from_str(&json_str).unwrap();
        Self::from_data(loaded.instructions, loaded.unknown)
    }

    fn from_data(instructions : Vec<Instruction>, unknown : Instruction) -> Self {
        let max = instructions.iter().map(|p| p.opcode).max().unwrap_or(0);

        let mut lookup : Vec<Instruction> = vec![unknown.clone(); (max as usize)+1];

        for i in instructions.iter() {
            lookup[i.opcode as usize] = i.clone();
        }

        for (i,o) in lookup.iter_mut().enumerate() {
            o.opcode = i as u16;
        }

        Self {
            lookup, instructions, unknown
        }
    }

    pub fn new() -> Self {
        let json_str = include_str!("src/emu/cpu/resources/opcodes.json");
        let loaded : Dbase = serde_json::from_str(json_str).unwrap();
        Self::from_data(loaded.instructions, loaded.unknown)
    }

    pub fn get(&self, opcode : u16) -> &Instruction {
        &self.lookup[opcode as usize]
    }

    pub fn all_instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "0x{:04x} => handle_op!({:?}, {}, 0x{:04x}, {}, {}),",
            self.opcode,
            self.addr_mode,
            self.action,
            self.opcode,
            self.cycles,
            self.size
            )
    }
}

impl fmt::Display for Dbase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", r#"
#[macro_export]
macro_rules! op_table {
    ($op:expr, $fail:block) => {
        match $op {"#)?;
        for i in self.instructions.iter() {
            writeln!(f, "\t\t{}", i)?
        }
        writeln!(f,"{}", r#"
            _ => $fail
        }
    }
}"#)
    }
}


