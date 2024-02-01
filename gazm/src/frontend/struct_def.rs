#![forbid(unused_imports)]
use crate::frontend::LabelDefinition;
use std::str::FromStr;

use crate::assembler::AssemblerCpuTrait;

use super::{ AstNodeKind,ParsedFrom };

#[derive(Debug, PartialEq, Clone)]
pub enum StructMemberType {
    Byte,
    Word,
    DWord,
    QWord,
    UserType(String),
}

impl FromStr for StructMemberType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ret = match s {
            "byte" => StructMemberType::Byte,
            "word" => StructMemberType::Word,
            "dword" => StructMemberType::DWord,
            "qword" => StructMemberType::QWord,
            _ => StructMemberType::UserType(s.to_string()),
        };

        Ok(ret)
    }
}

impl StructMemberType {
    pub fn to_size_item<C>(&self) -> AstNodeKind<C::NodeKind>
    where
        C: AssemblerCpuTrait,
    {
        use AstNodeKind::*;
        use ParsedFrom::Expression;
        match self {
            Self::Byte => Num(1, Expression),
            Self::Word => Num(2, Expression),
            Self::DWord => Num(4, Expression),
            Self::QWord => Num(8, Expression),
            Self::UserType(name) => Label(LabelDefinition::Text(format!("{name}.size"))),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructEntry {
    pub name: String,
    pub item_type: StructMemberType,
}
