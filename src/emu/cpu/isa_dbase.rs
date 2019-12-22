use serde::{Deserialize};
use serde::de::Deserializer;
// use serde::Deserializer;

#[derive(Debug, Clone,  Serialize, Deserialize)]
pub enum AddrMode {
    Indexed,
    Direct,
    Extended,
    Relative,
    Inherent,
    Immediate
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
#[serde(deserialize_with = "hex_str_to_num")]
    pub cycles : u16,
    pub opcode : String,
#[serde(deserialize_with = "usize_to_u16")]
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
        let table : Vec<Instruction> = serde_json::from_str(json_str).unwrap();
        let max = table.iter().map(|p| p.ins).max().unwrap_or(0);

        let mut instructions : Vec<Instruction> = vec![UNKNOWN.clone(); (max as usize)+1];

        for i in table.into_iter() {
            let op_code = i.ins;
            instructions[op_code as usize] = i;
        }

        instructions.into_iter().enumerate().map( |(n,mut i)| {
            i.ins = n as u16;
            i
        }).collect()
    };
}

pub fn unknown() -> &'static Instruction {
    &UNKNOWN
}

pub fn get(ins : usize) -> &'static Instruction {
    &INSTRUCTIONS[ins]
}

