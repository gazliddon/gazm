
use super::mem::{MemoryIO};
use super::cpu::{
    AddressLines, Direct, Extended, Immediate16, Immediate8, Indexed, Inherent,
    InstructionDecoder, Relative, Relative16, Regs
};

pub struct Dissembly {
    pub text: String,
    addr: u16,
    pub next_instruction_addr: u16,
    instruction: InstructionDecoder,
}

pub struct Disassembler<'a, M: 'a + MemoryIO> {
    mem: &'a M,
    regs: &'a Regs,
}

impl<'a,  M: 'a + MemoryIO> Disassembler<'a, M> {
    pub fn new(mem : &'a M, regs : &'a Regs) -> Self {
        Self {
            mem, regs
        }
    }

    fn diss_op<A: AddressLines>(&self, _ins: &mut InstructionDecoder) -> String {
        let _addr_mode = A::name();
        // effective_address = "EA";
        
        let action =&_ins.instruction_info.action;
        format!("{:<5}{}", action, A::diss(self.mem, _ins))
    }

    pub fn diss(&self, addr : u16) -> Dissembly {

        let mut ins = InstructionDecoder::new_from_inspect_mem(addr, self.mem);
        
        macro_rules! handle_op {
            ($addr:ident, $action:ident, $opcode:expr, $cycles:expr, $size:expr) => {{
                self.diss_op::<$addr>(&mut ins)
            }};
        }

        let text = op_table!(ins.instruction_info.opcode, { "".into() });

        let next_instruction_addr = ins.next_addr;

        Dissembly {
            text,
            addr,
            next_instruction_addr,
            instruction: ins,
        }
    }
}
