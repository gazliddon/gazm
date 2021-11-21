#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq, PartialOrd, Copy)]
pub enum BreakPointTypes {
    READ,
    WRITE,
    EXEC,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Copy)]
pub struct BreakPoint {
    addr: u16,
    bp_type: BreakPointTypes,
    active: bool,
    id: usize,
}

impl BreakPoint {
    /// Describes a breakpoint
    /// constructs defaulted to active
    pub fn new(bp_type: BreakPointTypes, addr: u16, id: usize) -> BreakPoint {
        BreakPoint {
            bp_type,
            addr,
            active: true,
            id,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn toggle_active(&mut self) {
        self.active = !self.active;
    }
}

#[derive(Clone, Debug)]
pub struct BreakPoints {
    break_points: std::collections::HashMap<usize, BreakPoint>,
    mem_to_bp: [Option<BreakPoint>; 0x1_0000],
    id: usize,
}

impl BreakPoints {
    pub fn new() -> BreakPoints {
        Self {
            break_points: std::collections::HashMap::new(),
            mem_to_bp: [None; 0x1_0000],
            id: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.break_points.len()
    }

    pub fn has_any_breakpoint(&self, addr: u16) -> bool {
        let bp = self.get_breakpoints(addr, 1);
        bp.iter().filter(|b| b.addr == addr).count() > 0
    }

    pub fn has_breakpoint(&self, addr: u16, bp_type: BreakPointTypes) -> bool {
        let bp = self.get_breakpoints(addr, 1);
        let num = bp
            .iter()
            .filter(|b| b.addr == addr && b.bp_type == bp_type)
            .count();
        num > 0
    }

    pub fn add(&mut self, addr: u16, bp_type: BreakPointTypes) -> Option<usize>{
        if !self.has_breakpoint(addr, bp_type) {
            let ret = self.id;
            let bp = BreakPoint::new(bp_type, addr, ret);
            self.id = self.id + 1;
            self.break_points.insert(ret, bp);
            Some(ret)
        } else {
            None
        }
    }
    pub fn find_breakpoint_id(&self, addr: u16, bp_type: BreakPointTypes) -> Option<usize> {
        self.find_breakpoint(addr, bp_type).map(|bp| bp.id)
    }

    fn find_breakpoint(&self, addr: u16, bp_type: BreakPointTypes) -> Option<&BreakPoint> {
        for (_, bp) in self.break_points.iter() {
            if bp.addr == addr && bp.bp_type == bp_type {
                return Some(bp);
            }
        }
        None
    }
    pub fn remove_by_id(&mut self, id : usize) {
        self.break_points.remove(&id);
    }

    pub fn remove(&mut self, addr: u16, bp_type: BreakPointTypes) {
        if let Some(id) = self.find_breakpoint_id(addr, bp_type) {
            self.break_points.remove(&id);
        }
    }

    fn get_range(addr: u16, range: usize) -> Option<(u16, u16)> {
        let last_addr = addr as usize + range - 1;
        if range > 1 && last_addr < 0x1_0000 {
            Some((addr, last_addr as u16))
        } else {
            None
        }
    }

    pub fn get_breakpoints<'a>(&'a self, addr: u16, range: usize) -> Vec<&'a BreakPoint> {
        if let Some((addr, last_addr)) = Self::get_range(addr, range) {
            self.break_points
                .values()
                .filter(|bp| bp.addr >= addr && bp.addr <= last_addr)
                .collect()
        } else {
            vec![]
        }
    }

    pub fn get_breakpoints_mut<'a>(
        &'a mut self,
        addr: u16,
        range: usize,
    ) -> Vec<&'a mut BreakPoint> {
        if let Some((addr, last_addr)) = Self::get_range(addr, range) {
            self.break_points
                .values_mut()
                .filter(|bp| bp.addr >= addr && bp.addr <= last_addr)
                .collect()
        } else {
            vec![]
        }
    }

    pub fn remove_all_at_addr(&mut self, addr: u16) {
        let v: Vec<usize> = self.get_breakpoints(addr, 1).iter().map(|b| b.id).collect();

        for id in v {
            self.break_points.remove(&id);
        }
    }
}
