use crate::postfix::{self, GetPriority, PostFixer};
use emu::utils::Stack;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Error)]
pub enum OperationErrorKind {
    #[error("Divide by zero")]
    DivideByZero,
    #[error("Overflow")]
    Overflow,
    #[error("Illegal bit operation")]
    IllegalBitOp,
    #[error("Illegal shift operation")]
    IllegalShift,
    #[error("Incompatible operands")]
    IncompatibleOperands,
    #[error("Illegal negation")]
    IllegalNegation,
}

pub type OperationError<T> = Result<T, OperationErrorKind>;


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

#[derive(Clone, Copy, Debug, PartialEq, Error)]
pub enum GenericEvalError {
    #[error("Expected an operator, got {0}")]
    UnexpectedOp(Operation),
    #[error("Unable to evaluate expression")]
    UnableToEvaluate,
    #[error("Did not expecte a value")]
    UnexpectedValue,
    #[error("Did not evaluate all items")]
    UnevaluatedTerms,
    #[error("Stack unexpectedly empty")]
    StackEmpty,
    #[error("Expected a value")]
    ExpectedValue,
    #[error("Expected an operator")]
    ExpectedOperator,
    #[error(transparent)]
    OperatorError(#[from] OperationErrorKind)

}

impl<T> From<OperationError<T>> for GenericEvalError {
    fn from(_: OperationError<T>) -> Self {
        todo!()
    }
}



pub trait EvalExpr<V: EvalTraits, I: ExprItemTraits<V> + Clone, ERR: From<GenericEvalError>> {
    fn eval_expr(&self, i: &I) -> Result<I, ERR>;
}

pub fn generic_postfix_eval<
    V: EvalTraits,
    I: ExprItemTraits<V> + Clone + From<V>,
    E: EvalExpr<V, I, ERR>,
    ERR: From<GenericEvalError>,
>(
    items: impl Iterator<Item = I>,
    evaluator: &E,
) -> Result<I, (usize, ERR)> {
    use Operation::*;
    use GenericEvalError::*;

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
                }.map(|v| I::from(v)).map_err(|e| (idx, GenericEvalError::from(e).into()))?
            }

            ExprItemKind::Value => i.clone(),
        };

        s.push(i)
    }

    match s.len() {
        0 => Err(StackEmpty),
        1 => Ok(s.pop()),
        _ => Err(UnevaluatedTerms),
    }
    .map_err(|e| idx_err(0, e))
}

#[derive(Clone, Debug, PartialEq)]
enum ExprItem<V: GetPriority> {
    Op(Operation),
    Val(V),
    Expr(Vec<ExprItem<V>>),
}

impl< V: GetPriority> ExprItem<V> {
    pub fn from_val(v : V) -> Self {
        ExprItem::Val(v)
    }

    pub fn from_op(op : Operation) -> Self {
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

mod test_common {}

mod value_test {
    use super::super::Value;
    use super::*;
    struct Evaluator {}
    impl GetPriority for Value {}
    type VOPVal = ExprItem<Value>;
    
    type OpValLocal = ExprItem<Value>;

    impl EvalTraits for Value {}


    impl EvalExpr<Value, OpValLocal, GenericEvalError> for Evaluator {
        fn eval_expr(&self, _i: &OpValLocal) -> Result<OpValLocal, GenericEvalError> {
            todo!()
        }
    }

    #[test]
    fn test_value() {
        use Value::*;
        use Operation::*;
        use ExprItem::*;
        let infix_items: Vec<OpValLocal> =
            vec![Val(Signed(10)), Op(Add), Val(Signed(20)), Op(Div), Val(Signed(5))];
        let mut pfix = PostFixer::new();
        let items = pfix.get_postfix(infix_items).unwrap();

        // let items: Vec<OpVal> = vec![10.into(), 10.into(), Add.into(), 5.into(), Div.into()];
        let evaluator = Evaluator {};
        let x = generic_postfix_eval(items.into_iter(), &evaluator);
        let desired = Ok(Val(Signed(14)));
        assert_eq!(x, desired);
    }
}

mod test {
    use super::*;

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
