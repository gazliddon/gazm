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

#[serde(deny_unknown_fields)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instruction {
    // pub display : Option<String>,
    pub addr_mode : AddrMode,

#[serde(deserialize_with = "u16::deserialize")]
    pub cycles : u16,
    pub action : String,
#[serde(deserialize_with = "hex_str_to_num")]
    pub opcode : u16,
#[serde(deserialize_with = "u16::deserialize")]
    pub size : u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dbase {
    unknown: Instruction,
    instructions: Vec<Instruction>,
#[serde(skip)]
    lookup: Vec<Instruction>
}

impl Dbase {

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
        let json_str = include_str!("resources/opcodes.json");
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


