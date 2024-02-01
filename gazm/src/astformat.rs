#![forbid(unused_imports)]


use crate::assembler::AssemblerCpuTrait;
use crate::frontend::AstNodeKind;
use crate::semantic::*;

impl<'a, C> std::fmt::Display for AstCtx<'a, C>
where
    C: AssemblerCpuTrait,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let wrapped = DisplayWrapper {
            node: self.get_tree().root(),
        };
        write!(f, "{wrapped}")
    }
}

struct DisplayWrapper<'a, C> 
where C: AssemblerCpuTrait
{
    node: AstNodeRef<'a, C>,
}

impl<'a, C> From<AstNodeRef<'a, C>> for DisplayWrapper<'a, C> 
where C: AssemblerCpuTrait

{
    fn from(ast: AstNodeRef<'a, C>) -> Self {
        Self { node: ast }
    }
}

pub fn as_string<C>(n: AstNodeRef<C>) -> String
where C: AssemblerCpuTrait
{
    let x = DisplayWrapper{node: n};
    x.to_string()
}

impl<'a, C> std::fmt::Display for DisplayWrapper<'a, C>
where C: AssemblerCpuTrait

{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AstNodeKind::*;

        let node = self.node;
        let item = &node.value().item;

        let to_string = |n: AstNodeRef<C>| -> String {
            let x: DisplayWrapper<C> = n.into();
            x.to_string()
        };

        let child_item = |n: usize| node.children().nth(n).map(|x| &x.value().item);

        let child_string = |n: usize| {
            if let Some(v) = node.children().nth(n) {
                to_string(v)
            } else {
                format!("ERR {:?}", node.value().item)
            }
        };

        let join_kids = |sep| {
            let v: Vec<_> = node.children().map(to_string).collect();
            v.join(sep)
        };

        let ret: String = match item {
            LocalAssignmentFromPc(name) => format!("!{name} equ *"),
            AssignmentFromPc(name) => format!("{name} equ *"),

            Pc => "*".to_string(),

            Label(name) => format!("{name}"),
            LocalLabel(name) => format!("!{name}"),
            Comment(comment) => format!("; {comment}"),

            // QuotedString(test) => format!("\"{}\"", test),
            // Register(r) => r.to_string(),
            // RegisterList(vec) => {
            //     let vec: Vec<_> = vec.iter().map(|r| r.to_string()).collect();
            //     vec.join(",")
            // }
            LocalAssignment(name) | Assignment(name) => {
                format!("{} equ {}", name, child_string(0))
            }

            Expr => join_kids(""),

            PostFixExpr => join_kids(" "),

            Include(file) => format!("include \"{}\"", file.to_string_lossy()),

            Num(n, _) => n.to_string(),
            // UnaryMinus => "-".to_string(),
            UnaryTerm => {
                format!("!{:?} {:?}", child_item(0), child_item(1))
            }

            Mul => "*".to_string(),
            Div => "/".to_string(),
            Add => "+".to_string(),
            Sub => "-".to_string(),
            BitAnd => "&".to_string(),
            BitOr => "|".to_string(),
            BitXor => "^".to_string(),

            Org => {
                format!("org {}", child_string(0))
            }

            Put => {
                format!("put {}", child_string(0))
            }

            BracketedExpr => {
                format!("({})", join_kids(""))
            }

            TokenizedFile(..) => {
                format!("TokFile:\n{}", join_kids("\n"))
            }


            StructDef(name) => {
                let body = join_kids(",\n");
                format!("struct {name} {{\n {body}\n}}")
            }

            StructEntry(name) => {
                format!("{name} : {}", child_string(0))
            }

            ShiftR => ">>".into(),

            ShiftL => "<<".into(),
            Fcc(text) => format!("{text:?}"),
            Fdb(_) | Fcb(_) => {
                format!("fcb {}", join_kids(","))
            }

            Fill => {
                let body = join_kids(",");
                format!("fill {body}")
            }

            MacroDef(name, vars) => {
                format!("macro {name} ({vars:?}) [{}]", join_kids(" : "))
            }

            CpuSpecific(..) => todo!(),
            // CpuSpecific(OpCode(_, instruction, amode)) => {
            //     use AddrModeParseType::*;

            //     let ind = |s: String, indirect: &bool| -> String {
            //         if *indirect {
            //             format!("[{s}]")
            //         } else {
            //             s
            //         }
            //     };

            //     let operand = match amode {
            //         RegisterSet => {
            //             let rset = &node.first_child().unwrap().value().item;
            //             if let CpuSpecific(MC6809::RegisterSet(regs)) = rset {
            //                 let r = regs
            //                     .iter()
            //                     .sorted()
            //                     .map(|r| r.to_string())
            //                     .collect_vec()
            //                     .join(",");
            //                 r.to_string()
            //             } else {
            //                 panic!("Whut!")
            //             }
            //         }
            //         Immediate => format!("#{}", child_string(0)),
            //         Direct => format!("<{}", child_string(0)),
            //         Extended(..) => child_string(0),
            //         Indexed(index_mode, indirect) => {
            //             use IndexParseType::*;
            //             match index_mode {
            //                 ConstantByteOffset(r, v) | Constant5BitOffset(r, v) => {
            //                     ind(format!("{v},{r}"), indirect)
            //                 }

            //                 ConstantWordOffset(r, v) => ind(format!("{v},{r}"), indirect),
            //                 PcOffsetWord(v) => ind(format!("{v},PC"), indirect),
            //                 PcOffsetByte(v) => ind(format!("{v},PC"), indirect),
            //                 ConstantOffset(r) => ind(format!("{},{r}", child_string(0)), indirect),
            //                 Zero(r) => ind(format!(",{r}"), indirect),
            //                 PreDecDec(r) => ind(format!(",--{r}"), indirect),
            //                 PreDec(r) => format!(",-{r}"),
            //                 PostIncInc(r) => ind(format!(",{r}++"), indirect),
            //                 PostInc(r) => format!(",{r}+"),
            //                 AddA(r) => ind(format!("A,{r}"), indirect),
            //                 AddB(r) => ind(format!("B,{r}"), indirect),
            //                 AddD(r) => ind(format!("D,{r}"), indirect),
            //                 PCOffset => ind(format!("{},PC", child_string(0)), indirect),
            //                 ExtendedIndirect => format!("[{}]", child_string(0)),
            //             }
            //         }
            //         _ => format!("{:?} NOT IMPLEMENTED", instruction.addr_mode),
            //     };

            //     format!("{} {operand}", instruction.action)
            // }

            _ => format!("{item:?} not implemented"),
        };

        write!(f, "{ret}")
    }
}
