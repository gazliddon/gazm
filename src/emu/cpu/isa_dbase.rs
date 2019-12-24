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

fn usize_to_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
where D: Deserializer<'de> {
    u16::deserialize(deserializer)
}

#[serde(deny_unknown_fields)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instruction {
    pub addr_mode : AddrMode,

#[serde(deserialize_with = "usize_to_u16")]
    pub cycles : u16,
    pub opcode : String,
#[serde(deserialize_with = "hex_str_to_num")]
    pub ins : u16,
#[serde(deserialize_with = "usize_to_u16")]
    pub size : u16,
}

lazy_static! {
    static ref UNKNOWN : Instruction = Instruction {
            addr_mode : AddrMode::Inherent,
            cycles : 0,
            ins : 0,
            opcode : ("???").into(),
            size : 1,
        };

    static ref INSTRUCTIONS : Vec<Instruction> = {
        let json_str = include_str!("resources/opcodes.json");

        let ret = serde_json::from_str(json_str);

        if !ret.is_ok() {
            println!("{:?}", ret)
        }

        ret.unwrap()
    };

    static ref INSTRUCTIONS_LOOKUP : Vec<Instruction> = {

        let max = INSTRUCTIONS.iter().map(|p| p.ins).max().unwrap_or(0);
        let mut lookup : Vec<Instruction> = vec![UNKNOWN.clone(); (max as usize)+1];

        for i in INSTRUCTIONS.iter() {
            let op_code = i.ins;
            lookup[op_code as usize] = i.clone();
        }

        for (i,o) in lookup.iter_mut().enumerate() {
            o.ins = i as u16;
        }

        lookup

    };
}

pub fn all_instructions() -> &'static Vec<Instruction> {
    &INSTRUCTIONS
}

pub fn get(ins : u16) -> &'static Instruction {
    &INSTRUCTIONS_LOOKUP[ins as usize]
}

