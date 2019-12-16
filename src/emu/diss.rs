use crate::mem::MemoryIO;

use crate::cpu::{RegEnum, IndexedFlags, IndexModes, InstructionDecoder, get_tfr_regs};

pub trait SymTab {
    fn get_symbol(&self, val : u16) -> Option<String>;

    fn get_symbol_with_default(&self, val : u16, def : &str) -> String {
        match self.get_symbol(val) {
            Some(text) => text,
            None => def.to_string()
        }
    }
}



#[derive(Default)]
pub struct Disassembler {
    text : String,
    is_upper_case : bool,
    hex_prefix: String,

}

impl Disassembler {

    pub fn new() -> Self {
        Disassembler { 
            is_upper_case : false,
            hex_prefix : "0x".to_string(),
            .. Default::default()
        }
    }

    fn add_op<M: MemoryIO>(&mut self, _m : &M, _diss: &mut InstructionDecoder, txt : &'static str) {
        self.text = format!("{:width$} {}", txt, self.text, width = 5);
    }

    fn expand<M : MemoryIO>(&mut self, _v : u16, def_str : &str, text : &'static str, _m: &M, _diss : &mut InstructionDecoder) {
        let op_str = String::from(text);

        // let def_str = match syms {
        //     &Some(tab) => tab.get_symbol_with_default(v, def_str),
        //     &None => def_str.clone(),
        // };
        //
        // Disassembly::new( &op_str.replace("OP", &def_str))
        
        self.text =  op_str.replace("OP", def_str);
    }

    fn text_from_byte_op<M : MemoryIO>(&mut self, text : &'static str, mem: &mut M, diss : &mut InstructionDecoder) { 

        let v = diss.fetch_byte(mem);
        let def_str  = format!("${:02X}", v);
        self.expand(u16::from(v), &def_str, text, mem, diss)
    }

    fn text_from_word_op<M : MemoryIO>(&mut self, text : &'static str, mem: &mut M, diss : &mut InstructionDecoder) { 
        let v = diss.fetch_word(mem);
        let def_str  = format!("${:04X}", v);
        self.expand(v, &def_str, text, mem, diss)
    }
}

fn stack_regs(op : u8 ) -> Vec<RegEnum>{

    let mut res = Vec::new();

    if (op & 0x80) == 0x80 {
        res.push(RegEnum::PC)
    }

    if ( op & 0x40 ) == 0x40  {
        res.push(RegEnum::S)
    }

    if ( op & 0x20 ) == 0x20  {
        res.push(RegEnum::Y)
    }

    if ( op & 0x10 ) == 0x10  {
        res.push(RegEnum::X)
    }

    if ( op & 0x8 ) == 0x8  {
        res.push(RegEnum::DP)
    }

    if ( op & 0x4 ) == 0x4  {
        res.push(RegEnum::B)
    }

    if ( op & 0x2 ) == 0x2  {
        res.push(RegEnum::A)
    }

    if ( op & 0x1 ) == 0x1  {
        res.push(RegEnum::CC)
    }

    res
}


fn tfr_regs(op : u8) -> Vec<RegEnum> {
    let (a,b) = get_tfr_regs(op);
    vec![a,b]
}

fn regs_to_str(byte : u8, f : fn(u8) -> Vec<RegEnum>) ->  String {

    let regs : Vec<String> = f(byte)
        .into_iter()
        .map(|x| format!("{:?}", x))
        .collect();

    regs.join(",")
} 

impl Disassembler {
    fn direct_8<M : MemoryIO>(&mut self, mem : &mut M, diss : &mut InstructionDecoder) { self.text_from_byte_op("<OP", mem,diss) }
    fn direct_16<M : MemoryIO>(&mut self, mem : &mut M, diss : &mut InstructionDecoder) { self.text_from_byte_op("<OP", mem,diss) }

    fn extended_16<M : MemoryIO>(&mut self, mem : &mut M, diss : &mut InstructionDecoder) { self.text_from_word_op("OP", mem, diss) }
    fn extended_8<M : MemoryIO>(&mut self, mem : &mut M, diss : &mut InstructionDecoder) { self.text_from_word_op("OP", mem, diss) }

    fn immediate8<M : MemoryIO>(&mut self, mem : &mut M, diss : &mut InstructionDecoder) { self.text_from_byte_op("#OP", mem, diss) }

    fn immediate16<M : MemoryIO>(&mut self, mem : &mut M, diss : &mut InstructionDecoder) { self.text_from_word_op("#OP", mem, diss) }

    fn inherent<M : MemoryIO>(&mut self, _mem : &mut M, _diss : &mut InstructionDecoder) { }

    fn inherent_reg_stack<M : MemoryIO>(&mut self, mem : &mut M, diss : &mut InstructionDecoder) { 
        let byte = diss.fetch_byte(mem);
        self.text = regs_to_str(byte,stack_regs);
    }

    fn inherent_reg_reg<M : MemoryIO>(&mut self, mem : &mut M, diss : &mut InstructionDecoder) { 
        let byte = diss.fetch_byte(mem);
        self.text = regs_to_str(byte,tfr_regs)
    }

    fn indexed_8<M : MemoryIO>(&mut self, mem : &mut M, diss : &mut InstructionDecoder) {
        self.indexed(mem,diss)
    }

    fn indexed_16<M : MemoryIO>(&mut self, mem : &mut M, diss : &mut InstructionDecoder) {
        self.indexed(mem,diss)
    }

    fn indexed<M : MemoryIO>(&mut self, mem : &mut M, diss : &mut InstructionDecoder) {

        let iflags = IndexedFlags::new(diss.fetch_byte(mem));

        let index_type = iflags.get_index_type();

        let mut s = match index_type {

            IndexModes::RPlus(r) => { 
                format!(",{:?}+",r)
            },

            IndexModes::RPlusPlus(r) => {
                format!(",{:?}++",r)
            },

            IndexModes::RSub(r) => {
                format!(",-{:?}",r) 
            },

            IndexModes::RSubSub(r) =>{
                format!(",--{:?}",r)
            },

            IndexModes::RZero(r) => { 
                format!(",{:?}",r) 
            },

            IndexModes::RAddB(r) => { 
                format!("B,{:?}", r) 
            },

            IndexModes::RAddA(r) => {
                format!("A,{:?}", r) 
            },

            IndexModes::RAddi8(r) => {
                format!("{},{:?}",diss.fetch_byte(mem) as i8, r)
            },

            IndexModes::RAddi16(r) => {
                format!("{},{:?}",diss.fetch_word(mem) as i16, r)
            },

            IndexModes::RAddD(r) => {
                format!("D,{:?}", r) 
            },

            IndexModes::PCAddi8 => {
                format!("PC,{:?}",diss.fetch_byte(mem) as i8)
            },

            IndexModes::PCAddi16 => {
                format!("PC,{:?}",diss.fetch_word(mem) as i16)
            },

            IndexModes::Illegal => { 
                "illegal".to_string() 
            },

            IndexModes::Ea=> {
                format!("${:04X}", diss.fetch_word(mem))
            },

            IndexModes::ROff(r,offset)=> {
                format!("{}, {:?}", offset as i16, r) 
            },
        };

        if iflags.is_indirect() {
            s = format!("[{}]", s);
        }

        self.text = s;
    }

    fn relative8<M : MemoryIO>(&mut self, mem : &mut M, diss : &mut InstructionDecoder) {
        let v = diss.fetch_byte(mem) as i8;
        let vstr = format!("{}", v);
        self.text = vstr;
    }

    fn relative16<M : MemoryIO>(&mut self, mem: &mut M, diss : &mut InstructionDecoder) {
        let v = diss.fetch_word(mem) as i16;
        let vstr = format!("{}", v);
        self.text = vstr;
    }
}

macro_rules! op {
    ($op:ident, $text:expr) => {
        fn $op<M: MemoryIO>(&mut self, mem : &mut M,  res : &mut InstructionDecoder) { self.add_op(mem, res, $text) }
    };
    ($op:ident) => {
        op!($op, stringify!($op));
    };
}

impl Disassembler {

    op!(neg);
    op!(abx);
    op!(adca);
    op!(adcb);
    op!(adda);
    op!(addb);
    op!(addd);
    op!(anda);
    op!(andb);
    op!(andcc);
    op!(asr);
    op!(asra);
    op!(asrb);
    op!(beq);
    op!(bge);
    op!(bgt);
    op!(bhi);
    op!(bhs_bcc);
    op!(bita);
    op!(bitb);
    op!(ble);
    op!(blo_bcs);
    op!(bls);
    op!(blt);
    op!(bmi);
    op!(bne);
    op!(bpl);
    op!(bra);
    op!(brn);
    op!(bsr);
    op!(bvc);
    op!(bvs);
    op!(clr);
    op!(clra);
    op!(clrb);
    op!(cmpa);
    op!(cmpb);
    op!(cmpx);
    op!(com);
    op!(coma);
    op!(comb);
    op!(cwai);
    op!(daa);
    op!(dec);
    op!(deca);
    op!(decb);
    op!(eora);
    op!(eorb);
    op!(exg);
    op!(inc);
    op!(inca);
    op!(incb);
    op!(jmp);
    op!(jsr);
    op!(lbra);
    op!(lbsr);
    op!(lda);
    op!(ldb);
    op!(ldd);
    op!(ldu);
    op!(ldx);
    op!(leas);
    op!(leau);
    op!(leax);
    op!(leay);
    op!(lsl_asl);
    op!(lsla_asla);
    op!(lslb_aslb);
    op!(lsr);
    op!(lsra);
    op!(lsrb);
    op!(mul);
    op!(nega);
    op!(negb);
    op!(nop);
    op!(ora);
    op!(orb);
    op!(orcc );
    op!(pshs);
    op!(pshu);
    op!(puls);
    op!(pulu);
    op!(reset);
    op!(rol);
    op!(rola);
    op!(rolb);
    op!(ror);
    op!(rora);
    op!(rorb);
    op!(rti);
    op!(rts);
    op!(sbca);
    op!(sbcb);
    op!(sex);
    op!(sta);
    op!(stb);
    op!(std);
    op!(stu);
    op!(stx);
    op!(suba);
    op!(subb);
    op!(subd);
    op!(swi);
    op!(sync);
    op!(tfr);
    op!(tst);
    op!(tsta);
    op!(tstb);
    op!(swi3);
    op!(cmpu);
    op!(cmps);
    op!(lbrn);
    op!(lbhi);
    op!(lbls);
    op!(lbhs_lbcc);
    op!(lblo_lbcs);
    op!(lbne);
    op!(lbeq);
    op!(lbvc);
    op!(lbvs);
    op!(lbpl);
    op!(lbmi);
    op!(lbge);
    op!(lblt);
    op!(lbgt);
    op!(swi2);
    op!(cmpd);
    op!(cmpy);
    op!(ldy);
    op!(lble);
    op!(sty);
    op!(lds);
    op!(sts);

    fn unimplemented(&mut self, _diss : &mut InstructionDecoder) {

    }

    pub fn diss<M: MemoryIO>(&mut self, mem : &mut M, addr : u16, _syms : Option<&dyn SymTab> ) -> (InstructionDecoder, String) {
        self.text = "".to_string();

        let mut diss = InstructionDecoder::new(addr);

        let op = diss.fetch_instruction(mem);

        decode_op!(op, self, mem, &mut diss);

        (diss, self.text.clone())
    }

}

