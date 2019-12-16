use super::mem::MemoryIO;
use super::{ Regs, InstructionDecoder, IndexModes, CpuErr};
use super::IndexedFlags;

pub trait AddressLines {

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

    fn diss_byte<M: MemoryIO>(_mem : &mut M,_regs : &mut Regs, _ins : &mut InstructionDecoder) -> String {
        "TBD".to_string()
    }

    fn diss_word<M: MemoryIO>(_mem : &mut M, _regs : &mut Regs, _ins : &mut InstructionDecoder) -> String {
        "TBD".to_string()
    }
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

    fn diss_byte<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> String {
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

    fn diss_byte<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> String {
        let val = ins.fetch_word(mem);
        format!("{:02x}", val)
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct Immediate { }

impl AddressLines for Immediate {
    fn name() -> String {
        "Immediate".to_string()
    }

    fn fetch_byte<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u8,CpuErr> {
        Ok( ins.fetch_byte(mem) )
    }

    fn fetch_word<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u16,CpuErr> {
        ins.add_cycles(1);
        Ok(ins.fetch_word(mem))
    }

    fn diss_byte<M: MemoryIO>(mem : &mut M, _regs : &mut Regs, ins : &mut InstructionDecoder) -> String {
        let val = ins.fetch_word(mem);
        format!("#{:02x}", val)
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


    fn diss_byte<M: MemoryIO>(_mem : &mut M, _regs : &mut Regs, _ins : &mut InstructionDecoder) -> String {
        "".to_string()
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct Indexed {}

impl Indexed { 

    fn get_index_mode<M: MemoryIO>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<(u16, IndexedFlags),CpuErr> {

        let index_mode_id = ins.fetch_byte(mem);

        let index_mode = IndexedFlags::new(index_mode_id) ;

        let itype = index_mode.get_index_type();

        match itype {
            IndexModes::RPlus(r) => { 
                // format!(",{:?}+",r)
                ins.add_cycles(3);
                let addr = regs.get(&r);
                regs.inc(&r);
                Ok(( addr,index_mode ))
            },

            IndexModes::RPlusPlus(r) => {
                ins.add_cycles(4);
                let addr = regs.get(&r);
                regs.incinc(&r);
                Ok(( addr,index_mode ))
            },

            IndexModes::RSub(r) => {
                ins.add_cycles(3);
                Ok((  regs.dec(&r),index_mode  ))
            },

            IndexModes::RSubSub(r) => {
                ins.add_cycles(4);
                Ok((  regs.decdec(&r), index_mode  ))
            },

            IndexModes::RZero(r) => { 
                ins.add_cycles(1);
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
                // format!("{},{:?}",diss.fetch_byte(mem) as i8, r)
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
                ins.add_cycles(6);
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

////////////////////////////////////////////////////////////////////////////////

// pub trait FetchWrite<M: MemoryIO> {
//     fn fetch<A: AddressLines>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<Self, CpuErr>;
//     fn store<A: AddressLines>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder, val : Self ) ;
// }

// impl<M : MemoryIO> FetchWrite<M> for u8 {
//     fn fetch<A: AddressLines>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u8,CpuErr> {
//         A::fetch_byte(mem, regs,ins)
//     }

//     fn store<A: AddressLines>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder, val : u8 )  {
//         A::store_byte(mem , regs , ins , val ) ;
//     }
// }

// impl<M : MemoryIO> FetchWrite<M> for u16 {
//     fn fetch<A: AddressLines>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder) -> Result<u16,CpuErr> {
//         A::fetch_word(mem, regs,ins)
//     }

//     fn store<A: AddressLines>(mem : &mut M, regs : &mut Regs, ins : &mut InstructionDecoder, val : u16 )  {
//         A::store_word(mem , regs , ins , val ) ;
//     }
// }


