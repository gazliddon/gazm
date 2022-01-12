use std::collections::{VecDeque, HashMap };
use std::hash::Hash;
use std::os::unix::prelude::OpenOptionsExt;

#[derive(Debug, PartialEq, Clone)]
pub struct Stack<OP: Clone > {
    stack : VecDeque<OP>,
}

impl<OP: Clone > Stack<OP> {
    pub fn new() -> Self {
        Self { stack: Default::default(), }
    }

    pub fn push(&mut self, op : OP) {
        self.stack.push_front(op)
    }

    pub fn pop(&mut self) -> OP {
        self.stack.pop_front().unwrap()
    }

    pub fn pop_pair(&mut self) -> (OP,OP) {
        let a = self.pop();
        let b = self.pop();
        (a,b)
    }

    pub fn flush(&mut self) -> Vec<OP> {
        let ret = self.stack.iter().cloned().collect();
        self.stack = VecDeque::new();
        ret
    }

    pub fn top(&self) -> Option<&OP> {
        self.stack.front()
    }
}

pub trait GetPriotity {
    fn priority(&self) -> Option<usize>;
    fn is_op(&self) -> bool {
        self.priority().is_some()
    }
}

#[derive(Debug, Clone)]
pub struct PostFixer<I : Clone + GetPriotity> {
    opstack : Stack<I>,
    input : Vec<I>,
    ret : Vec<I>,
}

impl<I : Clone + GetPriotity > PostFixer<I> {
    pub fn new() -> Self {
        Self {
            opstack: Stack::new(),
            input : Default::default(),
            ret : vec![],
        }
    }

    fn emit(&mut self, i : &I){
        self.ret.push(i.clone())
    }

    fn push(&mut self, op : &I){
        assert!(op.is_op());
        self.opstack.push(op.clone())
    }

    fn flush(&mut self) {
        let mut v = self.opstack.flush();
        self.ret.append(&mut v);
    }

    fn top_pri(&self) -> usize {
        if let Some(top_op) = self.opstack.top() {
            top_op.priority().unwrap()
        } else {
            0
        }
    }

    pub fn get_postfix(&mut self, ops : Vec<I>) ->Vec<I> {

        let mut it = ops.iter();

        if let Some(lhs)  = it.next() {
            self.emit(lhs);

            let mut first = true;
            let mut cit = it.as_slice().chunks(2);

            while let Some([ op,rhs ]) = cit.next() {
                if first {
                    self.emit(rhs);
                    self.push(op);
                    first = false;

                } else {
                    first = false;
                    self.emit(rhs);

                    let top_pri = self.top_pri();
                    let this_pri = op.priority().unwrap();

                    if this_pri > top_pri {
                        self.emit(op);
                        self.flush()
                    } else {
                        self.push(op);
                    }
                }
            }

            self.flush();

            std::mem::take(&mut self.ret)
        } else {
            ops
        }
    }
}


#[allow(unused_imports)]
mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    fn to_string(vs : &[char])-> String {
        String::from_iter(vs.iter())
    }

    impl GetPriotity for char {
        fn priority(&self) -> Option<usize> {
            match self {
                '/'=> Some(5),
                '*'=> Some(4),
                '+'=> Some(3),
                '-'=> Some(2),
                _ => None,
            }
        }
    }

    fn to_args(test : &str) -> (char, Vec<(char, char)>) {
        let mut it = test.chars();

        let first = it.next().unwrap();
        let mut rest = vec![];

        while let Some(op) = it.next() {
            let arg = it.next().unwrap();
            rest.push((op, arg))
        }
        (first, rest)
    }

    pub fn eval(e : &[ char ]) -> i64 {
        let mut s : Stack<i64> = Stack::new();

        let to_i64 = |c : char| {
            ( c as i64 ) - '0' as i64
        };

        for i in e.iter(){
            if i.is_op() {
                let (lhs, rhs) = s.pop_pair();
                let res = match i {
                    '*' => {println!("{} * {}", rhs, lhs); rhs * lhs },
                    '/' => {println!("{} / {}", rhs, lhs); rhs / lhs },
                    '+' => {println!("{} + {}", rhs, lhs); rhs + lhs },
                    '-' => {println!("{} - {}", rhs, lhs); rhs - lhs },
                    _ => panic!()
                };
                s.push(res);
            } else {
                s.push(to_i64(*i));
            }
        }

        s.pop()
    }

    #[test]
    fn postix() {
        let test =    "3+4*6-5-1";
        let desired = "346*+5-1-";

        println!("Input: {}", test);
        let mut x = PostFixer::new();

        let args : Vec<_> = test.chars().collect();

        let ret = x.get_postfix(args);
        let result = eval(&ret);
        let ret_str = to_string(&ret);

        println!("{:?}", test);
        println!("{:?}", ret_str);
        println!("{:?}", ret);
        assert_eq!(ret_str, desired);
        assert_eq!(result, 21);
    }
}

