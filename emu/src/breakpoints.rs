#[derive(Clone, Debug, PartialEq, PartialOrd, Copy)]
#[allow(dead_code)]
pub enum BreakPointTypes {
    READ,
    WRITE,
    EXEC,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Copy)]
#[allow(dead_code)]
pub struct BreakPoint {
    addr: u16,
    kind: BreakPointTypes,
}

#[allow(dead_code)]
impl BreakPoint {
    pub fn new(kind: BreakPointTypes, addr: u16) -> BreakPoint {
        BreakPoint { kind, addr }
    }

    pub fn new_read(addr: u16) -> BreakPoint {
        Self::new(BreakPointTypes::READ, addr)
    }

    pub fn new_write(addr: u16) -> BreakPoint {
        Self::new(BreakPointTypes::WRITE, addr)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub struct BreakPoints {
    break_points: Vec<BreakPoint>,
    mem_to_bp: [Option<BreakPoint>; 0x1_0000],
}

#[allow(dead_code)]
impl BreakPoints {
    pub fn new() -> BreakPoints {
        Self {
            break_points: vec![],
            mem_to_bp: [None; 0x1_0000],
        }
    }
    pub fn has_any_breakpoint(&self, addr: u16) -> bool {
        let a = self.has_exec_breakpoint(addr);
        let b = self.has_read_breakpoint(addr);
        let c = self.has_write_breakpoint(addr);

        a || b || c
    }

    pub fn has_breakpoint(&self, addr: u16, kind: BreakPointTypes) -> bool {
        let bp = BreakPoint { addr, kind };
        self.find(&bp).is_some()
    }

    pub fn has_exec_breakpoint(&self, addr: u16) -> bool {
        self.has_breakpoint(addr, BreakPointTypes::EXEC)
    }

    pub fn has_read_breakpoint(&self, addr: u16) -> bool {
        self.has_breakpoint(addr, BreakPointTypes::READ)
    }

    pub fn has_write_breakpoint(&self, addr: u16) -> bool {
        self.has_breakpoint(addr, BreakPointTypes::WRITE)
    }

    fn find(&self, b: &BreakPoint) -> Option<usize> {
        let mut it = self.break_points.iter();
        it.position(|bp| *bp == *b)
    }

    pub fn add(&mut self, b: &BreakPoint) {
        if self.find(b).is_some() {
            self.remove(b)
        }

        self.break_points.push(*b);
    }

    pub fn remove(&mut self, b: &BreakPoint) {
        if let Some(i) = self.find(b) {
            self.break_points.remove(i);
        }
    }

    pub fn remove_all_at_addr(&mut self, _addr: u16) {
        self.break_points = self
            .break_points
            .iter()
            .filter(|bp| bp.addr != _addr)
            .cloned()
            .collect();
    }
}
