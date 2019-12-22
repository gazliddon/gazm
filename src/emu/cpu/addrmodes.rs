use super::{ Regs, InstructionDecoder, IndexModes, CpuErr, IndexedFlags, mem};
use mem::MemoryIO;

pub trait AddressLines {

    fn diss<M: MemoryIO>(_mem : &mut M, _regs : &mut Regs, _ins : &mut InstructionDecoder) -> String {
        panic!("NOT IMP {:?}", Self::name())
    }

    fn ea<M: MemoryIO>(_mem : &mut M, _regs : &mut Regs, _ins : &mut InstructionDecoder) -> Result<u16,CpuErr> {
        warn!("EA for {}", Self::name());
        Err(CpuErr::IllegalAddressingMode)
    }

    fn store_byte<M: MemoryIO>(_mem : &mut M, _regs : &mut Regs, _ins : &mut InstructionDecoder, _val : u8 ) -> Result<u16,CpuErr>{
        Err(CpuErr::IllegalAddressingMode)
    }

    fn store_word<M: MemoryIO>(_mem : &mut M, _regs : &mut Regs, _ins : &mut InstructionDecoder, _val : u16 ) -> Result<u16,CpuErr> {
        Err(CpuErr::IllegalAddressingMode)
    }

    fn fetch_byte<M: MemoryIO>(_mem : &mut M, _regs : &mut Regs, _ins : &mut InstructionDecoder) -> Result<u8,CpuErr> {
        Err(CpuErr::IllegalAddressingMode)
    }

    fn fetch_word<M: MemoryIO>(_mem : &mut M, _regs : &mut Regs, _ins : &mut InstructionDecoder) -> Result<u16, CpuErr> {
        Err(CpuErr::IllegalAddressingMode)
    }

    fn fetch_byte_as_i16<M: MemoryIO>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<i16, CpuErr> {
        let byte = Self::fetch_byte(mem,regs,ins)? as i8;
        Ok(i16::from(byte))
    }

    fn name() -> String;
}


////////////////////////////////////////////////////////////////////////////////
pub struct Direct { }

impl AddressLines for Direct {

    fn ea<M: MemoryIO>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u16,CpuErr> {
        let index = u16::from(ins.fetch_byte(mem));
        Ok(regs.get_dp_ptr().wrapping_add(index))
    }

    fn name() -> String {
        "Direct".to_string()
    }

    fn fetch_byte<M: MemoryIO>( mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u8,CpuErr> {
        ins.add_cycles(2);
        let ea= Self::ea(mem,regs,ins)?;
        Ok(mem.load_byte(ea))
    }

    fn fetch_word<M: MemoryIO>( mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u16,CpuErr> {
        ins.add_cycles(3);
        let ea = Self::ea(mem,regs,ins)?;
        Ok(mem.load_word(ea))
    }

    fn store_byte<M: MemoryIO>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder, val : u8 ) -> Result<u16,CpuErr> {
        let ea = Self::ea(mem, regs, ins)?;
        mem.store_byte(ea,val);
        Ok(ea)
    }

    fn store_word<M: MemoryIO>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder, val : u16 )  -> Result<u16,CpuErr> {
        let ea = Self::ea(mem,regs,ins)?;
        mem.store_word(ea, val);
        Ok(ea)
    }

    fn diss<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> String {
        let val = ins.fetch_byte(mem);
        format!("<{:02x}", val)
    }
}


////////////////////////////////////////////////////////////////////////////////
pub struct Extended { }

impl AddressLines for Extended {
    fn ea<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u16,CpuErr>{
        ins.add_cycles(2);
        Ok(ins.fetch_word(mem))
    }

    fn name() -> String {
        "Extended".to_string()
    }

    fn fetch_byte<M: MemoryIO>( mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u8,CpuErr> {
        let addr = Self::ea(mem,regs,ins)?;
        ins.add_cycles(1);
        Ok( mem.load_byte(addr ))
    }

    fn fetch_word<M: MemoryIO>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u16,CpuErr> {
        let addr = Self::ea(mem,regs,ins)?;
        ins.add_cycles(2);
        Ok(mem.load_word(addr))
    }

    fn store_byte<M: MemoryIO>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder, val : u8 ) -> Result<u16,CpuErr> {
        let addr = Self::ea(mem,regs,ins)?;
        ins.add_cycles(1);
        mem.store_byte(addr, val);
        Ok(addr)
    }

    fn store_word<M: MemoryIO>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder, val : u16 ) -> Result<u16,CpuErr> {
        let addr = Self::ea(mem,regs,ins)?;
        ins.add_cycles(2);
        mem.store_word(addr, val);
        Ok(addr)
    }

    fn diss<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> String {
        let val = ins.fetch_word(mem);
        format!(">{:02x}", val)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Immediate8 { }

impl AddressLines for Immediate8 {
    fn name() -> String {
        "Immediate8".to_string()
    }

    fn fetch_byte<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u8,CpuErr> {
        Ok( ins.fetch_byte(mem) )
    }

    fn diss<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> String {
        let val = ins.fetch_byte(mem);
        format!("#{:02x}", val)
    }
}

pub struct Immediate16 { }

impl AddressLines for Immediate16 {
    fn name() -> String {
        "Immediate16".to_string()
    }

    fn fetch_word<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u16,CpuErr> {
        Ok( ins.fetch_word(mem) )
    }

    fn diss<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> String {
        let val = ins.fetch_word(mem);
        format!("#{:04x}", val)
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct Inherent { }

impl AddressLines for Inherent {
    fn name() -> String {
        "Inherent".to_string()
    }
    fn fetch_byte<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u8,CpuErr> {
        Ok(ins.fetch_byte(mem))
    }


    fn diss<M: MemoryIO>(_mem : &mut M, _regs : &mut Regs, _ins : &mut InstructionDecoder) -> String {
        "".to_string()
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct Indexed {}

impl Indexed { 
    fn get_cycles(itype : &IndexModes) -> u32 {
        use IndexModes::*;

        match itype {
            RPlus(_) => {0}
            RPlusPlus(_) => {0}
            RSub(_) => {0}
            RSubSub(_) => {0}
            RZero(_) => {0}
            RAddB(_) => {0}
            RAddA(_) => {0}
            RAddi8(_) => {0}
            RAddi16(_) => {0}
            RAddD(_) => {0}
            PCAddi8 => {0}
            PCAddi16 => {0}
            Illegal => {0}
            Ea=> {0}
            ROff(_,_)=> {0}
        }
    }

    fn add_cycles(itype : &IndexModes, ins : &mut InstructionDecoder) {
        ins.add_cycles(Self::get_cycles(itype));
    }


    fn get_index_mode<M: MemoryIO>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<(u16, IndexedFlags),CpuErr> {

        let index_mode_id = ins.fetch_byte(mem);

        let index_mode = IndexedFlags::new(index_mode_id) ;

        let itype = index_mode.get_index_type();

        Self::add_cycles(&itype, ins);

        match itype {
            IndexModes::RPlus(r) => { 
                // format!("{:?}+",r)
                let addr = regs.get(&r);
                regs.inc(&r);
                Ok(( addr,index_mode ))
            },

            IndexModes::RPlusPlus(r) => {
                // format!("{:?}++",r)
                let addr = regs.get(&r);
                regs.incinc(&r);
                Ok(( addr,index_mode ))
            },

            IndexModes::RSub(r) => {
                // format!("{:?}-",r)
                Ok((  regs.dec(&r),index_mode  ))
            },

            IndexModes::RSubSub(r) => {
                // format!("{:?}--",r)
                Ok((  regs.decdec(&r), index_mode  ))
            },

            IndexModes::RZero(r) => { 
                // format!("{:?}",r)
                Ok((  regs.get(&r), index_mode  ))
            },

            IndexModes::RAddB(r) => { 
                // format!("B,{:?}", r) 
                let add_r = u16::from(regs.b);
                Ok((  regs.get(&r).wrapping_add(add_r), index_mode  ))
            },

            IndexModes::RAddA(r) => {
                // format!("A,{:?}", r) 
                let add_r = u16::from(regs.a);
                Ok((  regs.get(&r).wrapping_add(add_r), index_mode  ))
            },

            IndexModes::RAddi8(r) => {
                // format!("{},{:?}",self.fetch_byte_as_i16(mem) as i8, r)
                let v = ins.fetch_byte_as_i16(mem) as u16;
                Ok((  regs.get(&r).wrapping_add(v), index_mode  ))
            },

            IndexModes::RAddi16(r) => {
                // format!("{},{:?}",diss.fetch_word(mem) as i16, r)
                let v = ins.fetch_word(mem);
                Ok((  regs.get(&r).wrapping_add(v), index_mode  ))
            },

            IndexModes::RAddD(r) => {
                // format!("D,{:?}", r) 
                let add_r = regs.get_d();
                Ok((  regs.get(&r).wrapping_add(add_r), index_mode  ))
            },

            IndexModes::PCAddi8 => {
                // format!("PC,{:?}",diss.fetch_byte(mem) as i8)
                let offset = ins.fetch_byte_as_i16(mem) as u16;
                Ok((  regs.pc.wrapping_add(offset), index_mode  ))
            },

            IndexModes::PCAddi16 => {
                // format!("PC,{:?}",diss.fetch_word(mem) as i16)
                let offset = ins.fetch_word(mem);
                Ok((  regs.pc.wrapping_add(offset), index_mode  ))
            },

            IndexModes::Illegal => { 
                Err(CpuErr::IllegalAddressingMode)
            },

            IndexModes::Ea=> {
                // format!("0x{:04X}", diss.fetch_word(mem))
                Ok((  ins.fetch_word(mem), index_mode  ))
            },

            IndexModes::ROff(r,offset)=> {
                // format!("{}, {:?}", offset, r) 
                Ok((  regs.get(&r).wrapping_add(offset), index_mode  ))
            },
        }
    }
}

impl AddressLines for Indexed {

    fn ea<M: MemoryIO>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u16,CpuErr> {
        ins.inc_cycles();

        let (ea,index_mode) = Indexed::get_index_mode::<M>(mem,regs, ins)?;

        let ea = if index_mode.is_indirect() {
            ins.add_cycles(3);
            mem.load_word(ea)
        }  else {
            ea
        };

        Ok(ea)
    }

    fn name() -> String {
        "Indexed".to_string()
    }

    fn fetch_byte<M: MemoryIO>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u8,CpuErr> {
        let ea = Self::ea(mem , regs , ins )?; 
        Ok(mem.load_byte(ea))
    }

    fn fetch_word<M: MemoryIO>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u16,CpuErr> {
        ins.inc_cycles();
        let ea = Self::ea(mem , regs , ins )?;
        Ok( mem.load_word(ea ))
    }

    fn store_byte<M: MemoryIO>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder, val : u8 ) -> Result<u16,CpuErr> {
        let ea = Self::ea(mem , regs , ins )?;
        mem.store_byte(ea, val);
        Ok(ea)
    }

    fn store_word<M: MemoryIO>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder, val : u16 ) -> Result<u16,CpuErr> {
        let ea = Self::ea(mem , regs , ins )?;
        mem.store_word(ea, val);
        Ok(ea)
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct Relative { }

impl AddressLines for Relative {
    fn name() -> String {
        "Relative".to_string()
    }

    fn fetch_byte<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u8,CpuErr> {
        Ok(ins.fetch_byte(mem))
    }

    fn fetch_word<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u16,CpuErr> {
        Ok(ins.fetch_word(mem))
    }
}

