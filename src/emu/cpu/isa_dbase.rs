use serde::{Deserialize};
use serde::de::Deserializer;

#[derive(Debug, Clone,  Serialize, Deserialize)]
pub enum AddrMode {
    TBD,
    Indexed,
    Direct,
    Extended,
    Relative,
    Inherent,
    Immediate
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Instruction {
    addr_mode : AddrMode,
    cycles : usize,
#[serde(deserialize_with = "hex_str_to_num")]
    ins : usize,
    opcode : String,
    size : usize,
}

pub struct Dbase {
    instructions: Vec<Instruction>,
}

impl Dbase {
    pub fn new() -> Self {
        let json_str = include_str!("resources/opcodes.json");
        let instructions : Vec<Instruction> = serde_json::from_str(json_str).unwrap();
        Self {
            instructions
        }
    }
}

fn hex_str_to_num<'de, D>(deserializer: D) -> Result<usize, D::Error>
where D: Deserializer<'de>,
{
    let hex_string = String::deserialize(deserializer)?;
    let z = usize::from_str_radix(&hex_string, 16);
    Ok(z.expect("Convert from hex str to usize"))
}

