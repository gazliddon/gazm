use super::{GenericEvalErrorKind, GetPriority, OperationError};
use crate::Stack;

/// Traits a value in an expression must support
pub trait EvalTraits:
    std::ops::Add<Output = OperationError<Self>>
    + std::ops::Sub<Output = OperationError<Self>>
    + std::ops::Mul<Output = OperationError<Self>>
    + std::ops::Div<Output = OperationError<Self>>
    + std::ops::Rem<Output = OperationError<Self>>
    + std::ops::BitOr<Output = OperationError<Self>>
    + std::ops::BitAnd<Output = OperationError<Self>>
    + std::ops::BitXor<Output = OperationError<Self>>
    + std::ops::Shl<Output = OperationError<Self>>
    + std::ops::Shr<Output = OperationError<Self>>
    + Sized
{
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExprItemKind {
    Expression,
    Value,
    Operator,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitOr,
    BitAnd,
    BitXor,
    ShiftRight,
    ShiftLeft,
}

impl GetPriority for Operation {
    fn priority(&self) -> Option<usize> {
        use Operation::*;
        let ret = match self {
            Div => 12,
            Rem => 12,
            Mul => 12,
            Add => 11,
            Sub => 11,
            ShiftRight => 10,
            ShiftLeft => 10,
            BitAnd => 9,
            BitXor => 8,
            BitOr => 7,
        };
        Some(ret)
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait ExprItemTraits<V: EvalTraits> {
    fn item_type(&self) -> ExprItemKind {
        use ExprItemKind::*;

        if self.is_value() {
            Value
        } else if self.is_expression() {
            Expression
        } else if self.is_op() {
            Operator
        } else {
            panic!()
        }
    }

    fn value(&self) -> Option<V>;
    fn op(&self) -> Option<Operation>;
    fn is_expression(&self) -> bool;

    fn is_op(&self) -> bool {
        self.op().is_some()
    }

    fn is_value(&self) -> bool {
        self.value().is_some()
    }
}

pub trait EvalExpr<V: EvalTraits, I: ExprItemTraits<V> + Clone, ERR: From<GenericEvalErrorKind>> {
    fn eval_expr(&self, i: &I) -> Result<I, ERR>;
}

pub fn evaluate_postfix_expr<
    V: EvalTraits,
    I: ExprItemTraits<V> + Clone + From<V>,
    E: EvalExpr<V, I, ERR>,
    ERR: From<GenericEvalErrorKind>,
>(
    items: impl Iterator<Item = I>,
    evaluator: &E,
) -> Result<V, (usize, ERR)> {
    use GenericEvalErrorKind::*;
    use Operation::*;

    // todo
    // check that we have enough items in the iterator?

    let mut s: Stack<I> = Stack::new();
    let idx_err = |idx, e| -> (usize, ERR) { (idx, ERR::from(e)) };

    for (idx, i) in items.enumerate() {
        let to_err = |e| -> (usize, ERR) { idx_err(idx, e) };

        let i = match i.item_type() {
            ExprItemKind::Expression => evaluator.eval_expr(&i).map_err(|e| (idx, e))?,

            ExprItemKind::Operator => {
                let op = i.op().ok_or(to_err(ExpectedOperator))?;

                let (rhs, lhs) = s.pop_pair();
                let lhs = lhs.value().ok_or(to_err(ExpectedValue))?;
                let rhs = rhs.value().ok_or(to_err(ExpectedValue))?;

                match op {
                    Mul => lhs * rhs,
                    Div => lhs / rhs,
                    Add => lhs + rhs,
                    Sub => lhs - rhs,
                    BitAnd => lhs & rhs,
                    BitXor => lhs ^ rhs,
                    BitOr => lhs | rhs,
                    ShiftLeft => lhs << rhs,
                    ShiftRight => lhs >> rhs,
                    Rem => lhs % rhs,
                }
                .map(|v| I::from(v))
                .map_err(|e| to_err(GenericEvalErrorKind::from(e)))?
            }

            ExprItemKind::Value => i.clone(),
        };

        s.push(i)
    }

    match s.len() {
        // Nothing on top of stack
        0 => Err(StackEmpty),
        // Something, try and extract the value
        1 => s.pop().value().ok_or(ExpectedValue),
        // Too many things on stack
        _ => Err(UnevaluatedTerms),
    }
    .map_err(|e| idx_err(0, e))
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprItem<V: GetPriority> {
    Op(Operation),
    Val(V),
    Expr(Vec<ExprItem<V>>),
}

impl<V: GetPriority> ExprItem<V> {
    pub fn from_val(v: V) -> Self {
        ExprItem::Val(v)
    }

    pub fn from_op(op: Operation) -> Self {
        ExprItem::Op(op)
    }
}

impl<V: GetPriority> GetPriority for ExprItem<V> {
    fn priority(&self) -> Option<usize> {
        use ExprItem::*;
        match self {
            Op(e) => e.priority(),
            Val(_) => None,
            Expr(_) => None,
        }
    }
}

impl<V: GetPriority + EvalTraits + Clone> ExprItemTraits<V> for ExprItem<V> {
    fn value(&self) -> Option<V> {
        match self {
            ExprItem::Val(v) => Some(v.clone()),
            _ => None,
        }
    }

    fn op(&self) -> Option<Operation> {
        match self {
            ExprItem::Op(v) => Some(*v),
            _ => None,
        }
    }

    fn is_expression(&self) -> bool {
        false
    }
}

impl<V: GetPriority> From<V> for ExprItem<V> {
    fn from(v: V) -> Self {
        ExprItem::Val(v)
    }
}

mod test {

    // type OpValLocal = ExprItem<isize>;

    // struct Evaluator {}

    // impl EvalExpr<isize, OpValLocal, GenericEvalError> for Evaluator {
    //     fn eval_expr(&self, _i: &OpValLocal) -> Result<OpValLocal, GenericEvalError> {
    //         todo!()
    //     }
    // }

    #[test]
    fn test_eval() {
        // use Operation::*;
        // use ExprItem::*;

        // let infix_items: Vec<OpValLocal> =
        //     vec![Val(10), Op(Add), Val(20), Op(Div), Val(5)];

        // let mut pfix = PostFixer::new();
        // let items = pfix.get_postfix(infix_items).unwrap();

        // // let items: Vec<OpVal> = vec![10.into(), 10.into(), Add.into(), 5.into(), Div.into()];
        // let evaluator = Evaluator {};
        // let x = generic_postfix_eval(items.into_iter(), &evaluator);
        // let desired = Ok(Val(14));
        // assert_eq!(x, desired);
    }
}
