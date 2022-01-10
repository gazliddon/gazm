use romloader::ResultExt;

use crate::postfix::{ Stack, GetPriotity };
use crate::item::Item;
use crate::ast::{ AstNodeMut, AstNodeRef,Ast, ItemWithPos, to_priority };

use std::{collections::HashMap, hash::Hash};

use crate::error::AstError;


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

    /// Evaluates a node in the AST tree the best it can
    /// Mutates the tree
    pub fn eval_node(&mut self, tree : &mut crate::ast::AstTree, id : crate::ast::AstNodeId) -> Result<(), AstError> {
        let node = tree.get(id).unwrap();

        let to_process : Vec<_> = node.descendants().map(|n| n.id()).collect();

        Ok(())
    }

    pub fn eval(&mut self, n : AstNodeRef) -> Result<Item, AstError> {
        use Item::*;

        let i = &n.value().item;

        let rez = match i {
            Assignment(name) => {
                let mut c  = n.children();

                let expr = c.next().ok_or_else(||
                    AstError::from_node("Bit weird",n)
                )?;

                let v = self.eval(expr)?;

                let num = v.get_number()
                    .ok_or_else(|| AstError::from_node("Didn't evaluate to a number", expr))?;

                self.syms.insert(name.clone(),num);
                v
            }

            PostFixExpr => self.eval_postfix(n)?,

            Label(name) => {
                let v = *self.syms.get(name)
                    .ok_or_else(|| 
                        AstError::from_node("Udefined symbol", n))?;
                Number(v)
            }

            Number(_) => i.clone(),

            _ => {
                return Err(AstError::from_node("Unable to evaluate ", n))
            }
        };

        Ok(rez)
    }

    pub fn eval_postfix(&mut self, n : AstNodeRef) -> Result<Item, AstError> {
        use Item::*;


        let mut items = vec![];

        for c in n.children() {
            let i = &c.value().item;
            let item = if i.is_op() {
                i.clone()
            } else {
                self.eval(c)?
            };

            items.push((c.clone(),item))
        }

        let mut s : Stack<Item> = Stack::new();

        for (cn,i) in items {
            if i.is_op() {
                let lhs = s.pop().get_number().unwrap();
                let rhs = s.pop().get_number().unwrap();
                let res = match i {
                    Mul => rhs * lhs,
                    Div => rhs / lhs,
                    Add => rhs + lhs,
                    Sub => rhs - lhs,
                    _ => return Err(AstError::from_node("Unexpected op ", cn)),
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



