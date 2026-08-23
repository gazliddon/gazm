#![forbid(unused_imports)]

use crate::cpu6809::frontend::{
    get_opcode_info, AddrModeParseType, IndexParseType, IndexWidth,
    NodeKind6809::{self, OpCode},
};

use emu6809::isa::{AddrModeEnum, InstructionId};

use crate::{
    assembler::{Assembler, ByteSize, ByteSizes, Sizer},
    error::GResult,
    semantic::AstNodeId,
};

fn size_indexed(
    sizer: &mut Sizer,
    asm: &mut Assembler,
    id: AstNodeId,
    pmode: IndexParseType,
    indirect: bool,
    ins: InstructionId,
) -> GResult<()> {
    {
        let current_scope_id = sizer.scopes.scope();
        let instruction = crate::cpu6809::assembler::ISA_DBASE.get_by_id(ins);

        sizer.advance_pc(instruction.size);
        use IndexParseType::*;

        match pmode {
            Zero(..) | AddA(..) | AddB(..) | AddD(..) | PostInc(..) | PostIncInc(..)
            | PreDec(..) | PreDecDec(..) => (),

            ConstantByteOffset(..)
            | PcOffsetByte(..)
            | PcOffsetWord(..)
            | ConstantWordOffset(..)
            | Constant5BitOffset(..) => {}

            ConstantOffset(r, width) => {
                let node = sizer.get_node(id);
                let (v, _) = asm.eval_first_arg(node, current_scope_id)?;

                let mut bs = match width {
                    IndexWidth::Auto => v.byte_size(),
                    IndexWidth::Bits5 if (-16..=15).contains(&v) => ByteSizes::Bits5(v as i8),
                    IndexWidth::Byte if (-128..=127).contains(&v) => ByteSizes::Byte(v as i8),
                    IndexWidth::Word if (-32768..=32767).contains(&v) => ByteSizes::Word(v as i16),
                    IndexWidth::Bits5 => {
                        return Err(crate::assembler::BinaryError::DoesNotFit {
                            dest_type: "5-bit indexed offset".to_string(),
                            val: v,
                        }
                        .into())
                    }
                    IndexWidth::Byte => {
                        return Err(crate::assembler::BinaryError::DoesNotFit {
                            dest_type: "8-bit indexed offset".to_string(),
                            val: v,
                        }
                        .into())
                    }
                    IndexWidth::Word => {
                        return Err(crate::assembler::BinaryError::DoesNotFit {
                            dest_type: "16-bit indexed offset".to_string(),
                            val: v,
                        }
                        .into())
                    }
                };

                if let ByteSizes::Bits5(val) = bs {
                    if indirect {
                        if width == IndexWidth::Bits5 {
                            return Err(crate::assembler::BinaryError::DoesNotFit {
                                dest_type: "5-bit indirect indexed offset".to_string(),
                                val: v,
                            }
                            .into());
                        }
                        // Indirect constant offset does not support
                        // 5 bit offsets so promote to 8 bit
                        bs = ByteSizes::Byte(val);
                    }
                }

                let new_amode = match bs {
                    ByteSizes::Zero => Zero(r),
                    ByteSizes::Bits5(v) => Constant5BitOffset(r, v),
                    ByteSizes::Word(v) => {
                        sizer.advance_pc(2);
                        ConstantWordOffset(r, v)
                    }
                    ByteSizes::Byte(v) => {
                        sizer.advance_pc(1);
                        ConstantByteOffset(r, v)
                    }
                };

                let new_item = OpCode(ins, AddrModeParseType::Indexed(new_amode, indirect));

                asm.add_fixup(id, new_item, current_scope_id);
            }

            PCOffset => {
                let node = sizer.get_node(id);
                let (v, _) = asm.eval_first_arg(node, current_scope_id)?;
                sizer.advance_pc(1);

                let new_amode = match v.byte_size() {
                    ByteSizes::Zero => PcOffsetByte(0),
                    ByteSizes::Bits5(v) | ByteSizes::Byte(v) => PcOffsetByte(v),
                    ByteSizes::Word(v) => {
                        sizer.advance_pc(1);
                        PcOffsetWord(v)
                    }
                };

                let new_item = OpCode(ins, AddrModeParseType::Indexed(new_amode, indirect));
                asm.add_fixup(id, new_item, current_scope_id);
            }

            ExtendedIndirect => {
                sizer.advance_pc(2);
            }
        };
        Ok(())
    }
}

pub fn size_node_internal(
    sizer: &mut Sizer,
    asm: &mut Assembler,
    id: AstNodeId,
    node_kind: NodeKind6809,
) -> GResult<()> {
    let current_scope_id = sizer.scopes.scope();
    let node = sizer.get_node(id);
    match &node_kind {
        NodeKind6809::SetDp => {
            let (dp, _) = asm.eval_first_arg(node, current_scope_id)?;
            if dp < 0 {
                panic!("Less than zerp?!?!?");
            }

            if dp >= 256 {
                panic!("Too big!")
            }

            asm.asm_out.set_dp(((dp as u64) & 0xff) as u8);
        }

        NodeKind6809::OpCode(ins, amode) => {
            let instruction = crate::cpu6809::assembler::ISA_DBASE.get_by_id(*ins);
            match amode {
                AddrModeParseType::Extended(false) => {
                    // If there is a direct page set AND
                    // I can evaluate the arg AND
                    // the instruction supports DIRECT addressing (phew)
                    // I can changing this to a direct page mode instruction
                    // !!!! and it wasn't forced (need someway to propogate this from parse)

                    let mut size = instruction.size;

                    let dp_info = get_opcode_info(*ins)
                        .and_then(|i_type| i_type.get_instruction(&AddrModeEnum::Direct))
                        .zip(asm.asm_out.direct_page);

                    if let Some((new_ins, dp)) = dp_info {
                        if let Ok((value, _)) = asm.eval_first_arg(node, current_scope_id) {
                            let top_byte = ((value >> 8) & 0xff) as u8;

                            if top_byte == dp {
                                // Here we go!
                                size = new_ins.size;
                                let new_item = OpCode(new_ins.id(), AddrModeParseType::Direct);
                                asm.add_fixup(id, new_item, current_scope_id);
                            }
                        }
                    }
                    sizer.advance_pc(size);
                }

                AddrModeParseType::Indexed(pmode, indirect) => {
                    size_indexed(sizer, asm, id, *pmode, *indirect, *ins)?;
                }

                _ => {
                    sizer.advance_pc(instruction.size);
                }
            };
        }

        _ => panic!(),
    }
    Ok(())
}
