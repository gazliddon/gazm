use romloader::ResultExt;

use crate::ast::AstNodeRef;

use super::ast::{ Ast, ItemWithPos, to_priority };
use std::{collections::HashMap, hash::Hash};
use super::item::Item;
use super::postfix::GetPriotity;

#[derive(Debug, Clone)]
pub struct Evaluator<'a> {
    children: Option<ego_tree::iter::Children<'a, ItemWithPos>>,
    syms: HashMap<String, i64>,
}

impl<'a> Evaluator<'a> {
    pub fn new() -> Self {
        Self {children : None, syms : Default::default()}
    }

    pub fn add_sym(&mut self, name : &str, val : i64) {
        self.syms.insert(name.into(),val);
    }

    pub fn eval(&mut self, n : &AstNodeRef) -> Result<Item, String> {
        use Item::*;

        let i = &n.value().item;

        let rez = match i {
            Assignment(name) => {
                let mut c  = n.children();
                let expr = c.next().ok_or("Assignment missing argument")?;

                let v = self.eval(&expr)?;
                let num = v.get_number().ok_or("Did not evaluate to a number")?;
                self.syms.insert(name.clone(),num);
                v
            }

            PostFixExpr => self.eval_postfix(n)?,

            Label(name) => {
                let v = *self.syms.get(name).ok_or_else(|| "Undefined label".to_string())?;
                Number(v)
            }

            Number(_) => i.clone(),

            _ => {return Err("Unable to eval!".to_string())}
        };

        Ok(rez)
    }

    pub fn eval_postfix(&mut self, n : &AstNodeRef) -> Result<Item, String> {
        use Item::*;
        use crate::postfix::Stack;
        use crate::ast::AstNodeRef;

        // go through all of the items and resolve to numbers

        let mut items: Vec<Item> = vec![];

        for c in n.children() {
            let i = &c.value().item;
            let item = if i.is_op() {
                i.clone()
            } else {
                self.eval(&c)?
            };
            items.push(item)
        }

        let mut s : Stack<Item> = Stack::new();

        for i in items {
            if i.is_op() {
                let lhs = s.pop().get_number().unwrap();
                let rhs = s.pop().get_number().unwrap();
                let res = match i {
                    Mul => rhs * lhs,
                    Div => rhs / lhs,
                    Add => rhs + lhs,
                    Sub => rhs - lhs,
                    _ => panic!()
                };
                s.push(Number(res))
            } else {
                s.push(i);
            }
        }

        Ok(s.pop())
    }

    fn take_num(&mut self) -> Option<i64> { 
        self.take_item()
            .and_then(|i| {
                match i {
                    Item::Number(n) => Some(n),
                    Item::Label(name) => self.syms.get(&name).cloned(),
                    _ => Some(0),
                }
            })
    }

    // fn take_pair(&mut self) -> Option<( Item, i64 )> { 
    //     let op = self.take_op();
    //     let num = self.take_num();

    //     if op.is_some() && num.is_some() {
    //         Some((op.unwrap(), num.unwrap()))
    //     } else {
    //         None
    //     }
    // }

    fn take_item(&mut self) -> Option<Item> {
        let children = self.children.as_mut().unwrap();
        children.next().map(|n| n.value().item.clone())
    }

    fn take_op(&mut self) -> Option<Item> {
        let to_op = |i| if to_priority(&i).is_some() {
            Some(i)
        } else {
            None
        };

        self.take_item().and_then(to_op)
    }
}



