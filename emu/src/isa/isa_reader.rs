#![allow(dead_code)]

use std::collections::HashMap;

use super::AddrModeEnum;
use std::fmt;

use serde::de::Deserializer;
use serde::Deserialize;
// use serde_derive::Deserialize;

// use std::collections::HashMap;

// Custome deserializers
fn hex_str_to_num<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let hex_string = String::deserialize(deserializer)?;
    let z = u16::from_str_radix(&hex_string, 16).expect("Convert from hex str to u16");
    Ok(z)
}

// Custome deserializers
fn fixup_action<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let action = String::deserialize(deserializer)?;
    Ok(action.to_lowercase().replace('/', "_"))
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub enum InstructionType {
    CallSubroutine,
    Return,
    Normal,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Instruction {
    // pub display : Option<String>,
    pub addr_mode: AddrModeEnum,
    #[serde(deserialize_with = "u16::deserialize")]
    pub cycles: u16,
    #[serde(deserialize_with = "fixup_action")]
    pub action: String,
    #[serde(deserialize_with = "hex_str_to_num")]
    pub opcode: u16,
    #[serde(deserialize_with = "u16::deserialize")]
    pub size: u16,
    #[serde(default)]
    #[serde(deserialize_with = "u16::deserialize")]
    pub operand_size: u16,
    pub subroutine: Option<bool>,

}

impl Instruction {
    pub fn as_macro(&self) -> String {
        format!(
            "0x{:04x} => handle_op!({:?}, {}, 0x{:04x}, {}, {}),",
            self.opcode, self.addr_mode, self.action, self.opcode, self.cycles, self.size
        )
    }
}

#[derive(Debug, Clone)]
pub struct InstructionInfo {
    pub mnemomic : String,
    pub ops: Vec<Instruction>,
    pub addressing_modes : std::collections::HashMap<AddrModeEnum, Instruction>,
}

impl InstructionInfo {
    pub fn new(i : Instruction) -> Self {
        let mut ret = Self {
            mnemomic: i.action.clone(),
            ops: vec![],
            addressing_modes: Default::default(),
        };
        ret.add(&i);
        ret
    }

    pub fn supports_addr_mode(&self, m : AddrModeEnum) -> bool {
        self.get_instruction(&m).is_some()
    }

    pub fn get_immediate_mode_supported(&self) -> Option<AddrModeEnum> {
        if self.supports_addr_mode(AddrModeEnum::Immediate8) {
            Some( AddrModeEnum::Immediate8 )
        } else if self.supports_addr_mode(AddrModeEnum::Immediate16) {
            Some( AddrModeEnum::Immediate16 )
        } else {
            None
        }
    }

    pub fn get_instruction(&self, amode : &AddrModeEnum) -> Option<&Instruction> {
        self.addressing_modes.get(amode)
    }

    pub fn add(&mut self, ins: &Instruction) {
        if self.addressing_modes.contains_key(&ins.addr_mode) {
            panic!("can't contain same addressing mode twice")
        }

        self.addressing_modes.insert(ins.addr_mode, ins.clone());
        self.ops.push(ins.clone());
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Dbase {
    unknown: Instruction,
    instructions: Vec<Instruction>,
    #[serde(skip)]
    lookup: Vec<Instruction>,
    #[serde(skip)]
    name_to_ins : HashMap<String, InstructionInfo>
}

fn split_opcodes(_input: &str) -> Option<(&str, &str)> {
    let split: Vec<&str> = _input.split('_').collect();

    if split.len() != 2 {
        None
    } else {
        Some((split[0], split[1]))
    }
}

impl Dbase {

    pub fn from_text(json_str : &str) -> Self {
        let loaded: Dbase = serde_json::from_str(json_str).unwrap();
        Self::from_data(loaded.instructions, loaded.unknown)
    }

    pub fn from_filename(file_name: &str) -> Self {
        let json_str = std::fs::read_to_string(file_name).unwrap();
        Self::from_text(&json_str)
    }

    fn from_data(instructions: Vec<Instruction>, unknown: Instruction) -> Self {
        let max = instructions.iter().map(|p| p.opcode).max().unwrap_or(0);

        let mut lookup: Vec<Instruction> = vec![unknown.clone(); (max as usize) + 1];

        for i in instructions.iter() {
            lookup[i.opcode as usize] = i.clone();
        }

        for (i, o) in lookup.iter_mut().enumerate() {
            o.opcode = i as u16;
        }

        let mut name_to_ins: HashMap<String, InstructionInfo> = HashMap::new();

        let mut add = |name: &str, i: &Instruction| {
            let i = i.clone();
            let name = String::from(name).to_ascii_lowercase();
            if let Some(rec) = name_to_ins.get_mut(&name) {
                rec.add(&i);
            } else {
                let info = InstructionInfo::new(i);
                name_to_ins.insert(name.to_string(), info);
            }
        };

        for i in &instructions {
            if let Some((a, b)) = split_opcodes(&i.action) {
                add(a, i);
                add(b, i);
            } else {
                add(&i.action, i);
            }
        }

        Self {
            lookup,
            instructions,
            unknown,
            name_to_ins,
        }
    }

    pub fn is_opcode(&self, input: &str) -> bool {
        self.get_opcode(input).is_some()
    }

    pub fn get_opcode(&self, input: &str) -> Option<&InstructionInfo> {
        let op = input.to_string().to_lowercase();
        self.name_to_ins.get(&op)
    }

    pub fn new() -> Self {
        Self::default()
            // let json_str = include_str!("../cpu/resources/opcodes.json");
            // let loaded: Dbase = serde_json::from_str(json_str).unwrap();
            // Self::from_data(loaded.instructions, loaded.unknown)
    }

    pub fn get(&self, opcode: u16) -> &Instruction {
        &self.lookup[opcode as usize]
    }

    pub fn all_instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }

}

impl Default for Dbase {
    fn default() -> Self {
        let json_str = include_str!("../cpu/resources/opcodes.json");
        let loaded: Dbase = serde_json::from_str(json_str).unwrap();
        Self::from_data(loaded.instructions, loaded.unknown)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "0x{:04x} => handle_op!({:?}, {}, 0x{:04x}, {}, {}),",
            self.opcode, self.addr_mode, self.action, self.opcode, self.cycles, self.size
            )
    }
}

impl fmt::Display for Dbase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header = 
            r#"#[macro_export]
macro_rules! op_table {
    ($op:expr, $fail:block) => {
        match $op {"#;

            let footer = 
                r#"
            _ => $fail
        }
    }
}"#;

writeln!( f, "{}", header )?;

for i in self.instructions.iter() {
    writeln!(f, "\t\t{}", i)?
}
writeln!( f, "{}",footer)
}
}
