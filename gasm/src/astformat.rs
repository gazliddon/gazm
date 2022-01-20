use crate::ast::*;
use crate::item::{self, IndexParseType, Item};
use std::fmt::Display;

impl std::fmt::Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let wrapped = DisplayWrapper {
            node: self.get_tree().root(),
        };
        write!(f, "{}", wrapped)
    }
}

pub fn join_vec<I: Display>(v: &[I], sep: &str) -> String {
    let ret: Vec<_> = v.iter().map(|x| x.to_string()).collect();
    ret.join(sep)
}

struct DisplayWrapper<'a> {
    node: AstNodeRef<'a>,
}

impl<'a> From<AstNodeRef<'a>> for DisplayWrapper<'a> {
    fn from(ast: AstNodeRef<'a>) -> Self {
        Self { node: ast }
    }
}

pub fn as_string(n: AstNodeRef) -> String {
    let x: DisplayWrapper = n.into();
    x.to_string()
}

impl<'a> std::fmt::Display for DisplayWrapper<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Item::*;

        let node = self.node;
        let item = &node.value().item;

        let to_string = |n: AstNodeRef| -> String {
            let x: DisplayWrapper = n.into();
            x.to_string()
        };

        let child = |n: usize| {
            let v = node.children().nth(n).unwrap();
            to_string(v)
        };

        let join_kids = |sep| {
            let v: Vec<_> = node.children().map(to_string).collect();
            v.join(sep)
        };

        let ret: String = match item {
            LocalAssignmentFromPc(name) | AssignmentFromPc(name) => {
                format!("{} equ {}", name, child(0))
            }

            Pc => "*".to_string(),

            Label(name) | LocalLabel(name) => name.clone(),

            Comment(comment) => comment.clone(),

            QuotedString(test) => format!("\"{}\"", test),
            // Register(r) => r.to_string(),
            RegisterList(vec) => {
                let vec: Vec<_> = vec.iter().map(|r| r.to_string()).collect();
                vec.join(",")
            }

            LocalAssignment(name) | Assignment(name) => {
                format!("{} equ {}", name, child(0))
            }

            Expr => join_kids(""),

            PostFixExpr => join_kids(" "),

            Include(file) => format!("include \"{}\"", file.to_string_lossy()),

            Number(n) => n.to_string(),
            UnaryMinus => "-".to_string(),
            UnaryTerm => {
                panic!()
            }

            Mul => "*".to_string(),
            Div => "/".to_string(),
            Add => "+".to_string(),
            Sub => "-".to_string(),
            And => "&".to_string(),
            Or => "|".to_string(),
            Xor => "^".to_string(),

            Org => {
                format!("org {}", child(0))
            }

            BracketedExpr => {
                format!("({})", join_kids(""))
            }

            TokenizedFile(_, _, _) => join_kids("\n"),

            OpCode(ins, item::AddrModeParseType::Inherent) => ins.action.clone(),

            OpCode(ins, amode) => {
                use item::AddrModeParseType::*;

                let ind = |s: String, indirect: &bool| -> String {
                    if *indirect {
                        format!("[{}]", s)
                    } else {
                        s
                    }
                };

                let operand = match amode {
                    Immediate => format!("#{}", child(0)),
                    Direct => format!("<{}", child(0)),
                    Indexed(imode) => {
                        use item::IndexParseType::*;
                        match imode {
                            ConstantByteOffset(r, v, indirect)
                            | Constant5BitOffset(r, v, indirect) => {
                                ind(format!("{},{}", v, r), indirect)
                            }

                            ConstantWordOffset(r, v, indirect) => {
                                ind(format!("{},{}", v, r), indirect)
                            }

                            PcOffsetWord(v, indirect) => ind(format!("{},PC", v), indirect),
                            PcOffsetByte(v, indirect) => ind(format!("{},PC", v), indirect),
                            ConstantOffset(r, indirect) => {
                                ind(format!("{},{}", child(0), r), indirect)
                            }
                            Zero(r, indirect) => ind(format!(",{}", r), indirect),
                            SubSub(r, indirect) => ind(format!(",--{}", r), indirect),
                            Sub(r) => format!(",-{}", r),
                            PlusPlus(r, indirect) => ind(format!(",{}++", r), indirect),
                            Plus(r) => format!(",{}+", r),
                            AddA(r, indirect) => ind(format!("A,{}", r), indirect),
                            AddB(r, indirect) => ind(format!("B,{}", r), indirect),
                            AddD(r, indirect) => ind(format!("D,{}", r), indirect),
                            PCOffset(indirect) => ind(format!("{},PC", child(0)), indirect),
                            ExtendedIndirect => format!("[{}]", child(0)),
                        }
                    }
                    _ => format!("{:?} NOT IMPLEMENTED", ins.addr_mode),
                };

                format!("{} {}", ins.action, operand)
            }
            _ => format!("{:?} not implemented", item),
        };

        write!(f, "{}", ret)
    }
}
