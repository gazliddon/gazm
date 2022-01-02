use std::collections::{VecDeque, HashMap };
use std::hash::Hash;

#[derive(Debug, PartialEq, Clone)]
pub struct OStack<OP: PartialEq + Eq + Clone + Hash> {
    stack : VecDeque<OP>,
    pri : HashMap<OP, usize>,
}

impl<OP: PartialEq + Eq + Clone + Hash> OStack<OP> {
    pub fn new(pri : HashMap<OP,usize>) -> Self {
        Self {
            stack: Default::default(),
            pri
        }
    }

    pub fn push(&mut self, op : OP) {
        self.stack.push_front(op)
    }

    pub fn pop(&mut self) -> OP {
        self.stack.pop_front().unwrap()
    }

    pub fn flush(&mut self) -> Vec<OP> {
        let ret = self.stack.iter().cloned().collect();

        self.stack = VecDeque::new();

        ret
    }

    pub fn get_pri(&self, c : &OP)  -> usize {
        if let Some(p) = self.pri.get(&c) {
            *p
        } else {
            todo!("Make these functions return errors on wrong types");
        }
    }
    pub fn top_pri(&self) -> usize {
        assert!(!self.stack.is_empty());
        let top = self.stack.front().unwrap();
        self.get_pri(top)
    }
}

#[derive(Debug, Clone)]
pub enum InfixItem<I,OP> {
    Item(I),
    Op(OP)
}

#[derive(Debug, Clone)]
pub struct InFixer<I : Clone+ PartialEq + Eq,OP: PartialEq + Eq + Clone + Hash> {
    opstack : OStack<OP>,
    input : Vec<InfixItem<I,OP>>,
    ret : Vec<InfixItem<I,OP>>,

}

impl<I : Clone+ PartialEq + Eq,OP: PartialEq + Eq + Clone + Hash> InFixer<I, OP> {
    pub fn new(pri : HashMap<OP,usize> ) -> Self {
        Self {
            opstack: OStack::new(pri),
            input : Default::default(),
            ret : vec![]
        }
    }

    fn append(&mut self, i : &InfixItem<I,OP>) {
        self.ret.push(i.clone())
    }

    fn append_op_vec(&mut self, ov : Vec<OP>) {
        let mut extra = ov.into_iter().map(|o| InfixItem::Op(o)).collect();
        self.ret.append(&mut extra);
    }

    fn push_op(&mut self, i : &InfixItem<I, OP>) {
        match i {
            InfixItem::Op(op) => self.opstack.push(op.clone()),
            _=> todo!()
        }
    }

    fn flush(&mut self) {
        let v = self.opstack.flush();
        self.append_op_vec(v);
    }
    
    fn get_pri(&mut self, i : &InfixItem<I, OP>) -> usize {
        match i {
            InfixItem::Op(op) => self.opstack.get_pri(&op),
            _=> todo!()
        }
    }

    fn top_pri(&mut self) -> usize {
        self.opstack.top_pri()
    }

    pub fn to_infix(&mut self, a : Vec<InfixItem<I,OP>>) ->Vec<InfixItem<I,OP>> {

        if a.len() < 3 {
            return a
        }

        let mut it = a.iter();

        let a = it.next().unwrap();
        let op = it.next().unwrap();

        self.append(a);
        self.push_op(op);

        while let Some(c) = it.next() {
            self.append(c);
            let op = it.next();
            if op.is_none() {
                break;
            }
            let op = op.unwrap();
            let op_pri = self.get_pri(&op);
            let top_pri = self.top_pri();

            if top_pri > op_pri {
                self.flush()
            } 
            self.push_op(op)
        }

        self.flush();

        std::mem::replace(&mut self.ret, vec![])
    }
}

