// use crate::eval::OperatorTraits;

use crate::eval::OperationError;
use crate::eval::OperationErrorKind;

#[derive(Debug, PartialEq, Clone)]
pub enum EvalResultType {}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Null,
    Macro,
    Signed(i64),
    Unsigned(u64),
    Text(String),
    Float(f64),
}

impl<T: Into<String>> From<T> for Value {
    fn from(x: T) -> Self {
        Value::Text(x.into())
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value::*;

        let x = match self {
            Signed(a) => format!("{a}i64"),
            Unsigned(a) => format!("{a}u64"),
            Float(a) => format!("{a}f64"),
            Text(a) => format!("\"{a}\""),
            Macro => "macro".to_string(),
            Null => "null".to_string(),
        };

        f.write_str(&x)
    }
}

impl Value {
    pub fn is_number(&self) -> bool {
        use Value::*;
        match self {
            Unsigned(_) | Signed(_) | Float(_) => true,
            _ => false,
        }
    }

    pub fn as_double(self) -> Self {
        use Value::*;
        match self {
            Signed(a) => Float(a as f64),
            Unsigned(a) => Float(a as f64),
            Float(_) => self,
            _ => Null,
        }
    }

    pub fn as_unsigned(self) -> Self {
        use Value::*;
        match self {
            Signed(a) => Unsigned(a as u64),
            Unsigned(_) => self,
            Float(a) => Unsigned(a as u64),
            _ => Null,
        }
    }

    pub fn as_signed(self) -> Self {
        use Value::*;
        match self {
            Signed(_) => self,
            Unsigned(a) => Signed(a as i64),
            Float(a) => Signed(a as i64),
            _ => Null,
        }
    }

    pub fn as_text(self) -> Self {
        Value::Text(format!("{self}"))
    }
}

// signed + unsigned = unsigned
// signed + double = double
impl std::ops::Add for Value {
    type Output = OperationError<Value>;

    fn add(self, rhs: Value) -> Self::Output {
        use OperationErrorKind::*;
        use Value::*;
        match (self, &rhs) {
            (Signed(a), Signed(b)) => Ok(Signed(a + b)),
            (Signed(a), Unsigned(b)) => Ok(Signed(a + *b as i64)),
            (Signed(a), Float(b)) => Ok(Float(a as f64 + b)),
            (Unsigned(a), Signed(b)) => Ok(Signed(a as i64 + b)),
            (Unsigned(a), Unsigned(b)) => Ok(Unsigned(a + b)),
            (Unsigned(a), Float(b)) => Ok(Float(a as f64 + b)),
            (Float(a), Signed(b)) => Ok(Float(a + *b as f64)),
            (Float(a), Unsigned(b)) => Ok(Float(a + *b as f64)),
            (Float(a), Float(b)) => Ok(Float(a + *b as f64)),
            (Text(a), Text(b)) => Ok(Text(a + b)),

            (Macro, _) | (_, Macro) | (Text(_), _) | (_, Text(_)) | (_, Null) | (Null, _) => {
                Err(IncompatibleOperands)
            }
        }
    }
}

impl std::ops::Sub for Value {
    type Output = OperationError<Value>;

    fn sub(self, rhs: Value) -> Self::Output {
        use Value::*;
        use OperationErrorKind::*;
        match (self, &rhs) {
            (Signed(a), Signed(b)) => Ok( Signed(a - b) ),
            (Signed(a), Unsigned(b)) => Ok( Signed(a - *b as i64) ),
            (Signed(a), Float(b)) => Ok( Float(a as f64 - b) ),
            (Unsigned(a), Signed(b)) => Ok( Signed(a as i64 - b) ),
            (Unsigned(a), Unsigned(b)) => Ok( Unsigned(a - b) ),
            (Unsigned(a), Float(b)) => Ok( Float(a as f64 - b) ),
            (Float(a), Signed(b)) => Ok( Float(a - *b as f64) ),
            (Float(a), Unsigned(b)) => Ok( Float(a - *b as f64) ),
            (Float(a), Float(b)) => Ok( Float(a - *b as f64) ),
            (Null, _) |
            (_, Null) |
            (Macro, _) |
            (_, Macro) |
            (Text(_), _) |
            (_, Text(_)) =>
                Err(IncompatibleOperands)
        }
    }
}
impl std::ops::Mul for Value {
    type Output = OperationError<Value>;

    fn mul(self, rhs: Value) -> Self::Output {

        use Value::*;
        use OperationErrorKind::*;

        match (self, &rhs) {
            (Signed(a), Signed(b)) => Ok( Signed(a * b) ),
            (Signed(a), Unsigned(b)) => Ok( Signed(a * *b as i64) ),
            (Signed(a), Float(b)) => Ok( Float(a as f64 * b) ),
            (Unsigned(a), Signed(b)) => Ok( Signed(a as i64 * b) ),
            (Unsigned(a), Unsigned(b)) => Ok( Unsigned(a * b) ),
            (Unsigned(a), Float(b)) => Ok( Float(a as f64 * b) ),
            (Float(a), Signed(b)) => Ok( Float(a * *b as f64) ),
            (Float(a), Unsigned(b)) => Ok( Float(a * *b as f64) ),
            (Float(a), Float(b)) => Ok( Float(a * *b as f64) ),
            (Null, _) |
            (_, Null) |
            (Macro, _) |
            (_, Macro) |
            (Text(_), _) |
            (_, Text(_)) =>
                Err(IncompatibleOperands)
        }
    }
}

impl std::ops::Div for Value {
    type Output = OperationError<Value>;

    fn div(self, rhs: Value) -> Self::Output {

        use Value::*;
        use OperationErrorKind::*;

        match (self, &rhs) {
            (Signed(a), Signed(b)) => Ok( Signed(a / b) ),
            (Signed(a), Unsigned(b)) => Ok( Signed(a / *b as i64) ),
            (Signed(a), Float(b)) => Ok( Float(a as f64 / b) ),
            (Unsigned(a), Signed(b)) => Ok( Signed(a as i64 / b) ),
            (Unsigned(a), Unsigned(b)) => Ok( Unsigned(a / b) ),
            (Unsigned(a), Float(b)) => Ok( Float(a as f64 / b) ),
            (Float(a), Signed(b)) => Ok( Float(a / *b as f64) ),
            (Float(a), Unsigned(b)) => Ok( Float(a / *b as f64) ),
            (Float(a), Float(b)) => Ok( Float(a / *b as f64) ),
            (Null, _) |
            (_, Null) |
            (Macro, _) |
            (_, Macro) |
            (Text(_), _) |
            (_, Text(_)) =>
                Err(IncompatibleOperands)
        }
    }
}

impl std::ops::Rem for Value {
    type Output = OperationError<Value>;

    fn rem(self, rhs: Value) -> Self::Output {
        use Value::*;
        use OperationErrorKind::*;

        match (self, &rhs) {
            (Signed(a), Signed(b)) => Ok( Signed(a % b) ),
            (Signed(a), Unsigned(b)) => Ok( Signed(a % *b as i64) ),
            (Signed(a), Float(b)) => Ok( Float(a as f64 % b) ),
            (Unsigned(a), Signed(b)) => Ok( Signed(a as i64 % b) ),
            (Unsigned(a), Unsigned(b)) => Ok( Unsigned(a % b) ),
            (Unsigned(a), Float(b)) => Ok( Float(a as f64 % b) ),
            (Float(a), Signed(b)) => Ok( Float(a % *b as f64) ),
            (Float(a), Unsigned(b)) => Ok( Float(a % *b as f64) ),
            (Float(a), Float(b)) => Ok( Float(a % *b as f64) ),
            (Null, _) |
            (_, Null) |
            (Macro, _) |
            (_, Macro) |
            (Text(_), _) |
            (_, Text(_)) =>
                Err(IncompatibleOperands)
        }
    }
}

impl std::ops::BitXor for Value {
    type Output = OperationError<Self>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        use Value::*;
        use OperationErrorKind::*;
        match (self, &rhs) {
            (Signed(a), Signed(b)) => Ok( Signed(a ^ b) ),
            (Signed(a), Unsigned(b)) => Ok( Signed(a ^ *b as i64) ),
            (Unsigned(a), Signed(b)) => Ok( Signed(a as i64 ^ *b)),
            (Unsigned(a), Unsigned(b)) => Ok( Unsigned(a ^ b) ),
            (Float(_), _) |
            (_, Float(_)) |
            (Null, _) |
            (_, Null) |
            (Macro, _) |
            (_, Macro) |
            (Text(_), _) |
            (_, Text(_)) => 
                Err(IncompatibleOperands)
        }
    }
}
impl std::ops::BitAnd for Value {
    type Output = OperationError<Self>;

    fn bitand(self, rhs: Self) -> Self::Output {
        use Value::*;
        use OperationErrorKind::*;
        match (self, &rhs) {
            (Signed(a), Signed(b)) => Ok( Signed(a & b) ),
            (Signed(a), Unsigned(b)) => Ok( Signed(a & *b as i64) ),
            (Unsigned(a), Signed(b)) => Ok( Signed(a as i64 & *b)),
            (Unsigned(a), Unsigned(b)) => Ok( Unsigned(a & b) ),
            (Float(_), _) |
            (_, Float(_)) |
            (Null, _) |
            (_, Null) |
            (Macro, _) |
            (_, Macro) |
            (Text(_), _) |
            (_, Text(_)) => 
                Err(IncompatibleOperands)
        }
    }
}
impl std::ops::BitOr for Value {
    type Output = OperationError<Self>;

    fn bitor(self, rhs: Self) -> Self::Output {
        use Value::*;
        use OperationErrorKind::*;
        match (self, &rhs) {
            (Signed(a), Signed(b)) => Ok( Signed(a | b) ),
            (Signed(a), Unsigned(b)) => Ok( Signed(a | *b as i64) ),
            (Unsigned(a), Signed(b)) => Ok( Signed(a as i64 | b)),
            (Unsigned(a), Unsigned(b)) => Ok( Unsigned(a | b) ),
            (Float(_), _) |
            (_, Float(_)) |
            (Null, _) |
            (_, Null) |
            (Macro, _) |
            (_, Macro) |
            (Text(_), _) |
            (_, Text(_)) => 
                Err(IncompatibleOperands)
        }
    }
}
impl std::ops::Shr for Value {
    type Output = OperationError<Self>;

    fn shr(self, rhs: Self) -> Self::Output {
        use Value::*;
        use OperationErrorKind::*;
        match (self, &rhs) {
            (Signed(a), Signed(b)) => Ok( Signed(a >> b) ),
            (Signed(a), Unsigned(b)) => Ok( Signed(a >> *b as i64) ),
            (Unsigned(a), Signed(b)) => Ok( Signed(( a as i64 ) >> b)),
            (Unsigned(a), Unsigned(b)) => Ok( Unsigned(a >> b) ),
            (Float(_), _) |
            (_, Float(_)) |
            (Null, _) |
            (_, Null) |
            (Macro, _) |
            (_, Macro) |
            (Text(_), _) |
            (_, Text(_)) => 
                Err(IncompatibleOperands)
        }
    }
}

impl std::ops::Shl for Value {
    type Output = OperationError<Self>;

    fn shl(self, rhs: Self) -> Self::Output {
        use Value::*;
        use OperationErrorKind::*;
        match (self, &rhs) {
            (Signed(a), Signed(b)) => Ok( Signed(a << b) ),
            (Signed(a), Unsigned(b)) => Ok( Signed(a << *b as i64) ),
            (Unsigned(a), Signed(b)) => Ok( Signed((a as i64) << b)),
            (Unsigned(a), Unsigned(b)) => Ok( Unsigned(a << b) ),
            (Float(_), _) |
            (_, Float(_)) |
            (Null, _) |
            (_, Null) |
            (Macro, _) |
            (_, Macro) |
            (Text(_), _) |
            (_, Text(_)) => 
                Err(IncompatibleOperands)
        }
    }
}

impl std::ops::Neg for Value {
    type Output = OperationError<Self>;

    fn neg(self) -> Self::Output {
        // use OperationErrorKind::*;
        use Value::*;
        match self {
            Signed(a) => Ok( Signed(-a )),
            Unsigned(a) => Ok( Signed(-(a as i64 ))),
            Float(a) => Ok( Float(-a )),
            _ => Err(OperationErrorKind::IllegalNegation),
        }
    }
}
#[allow(unused_imports)]

mod test_postfix {}

#[allow(unused_imports)]
mod test {

    use super::*;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

    #[test]
    fn test_vals() {
        use Value::*;

        let a = 10;
        let b = 20;

        let v1 = Unsigned(a);
        let v2 = Unsigned(b);
        assert_eq!(v1 * v2, Ok(Unsigned(a * b)));

        let v1 = Float(a as f64);
        let v2 = Unsigned(b);
        assert_eq!(v2.clone() * v1.clone(), Ok(Float((a * b) as f64)));
        assert_eq!(v1 * v2, Ok( Float((a * b) as f64) ));

        let v1 = Float(a as f64);
        let v2 = Unsigned(b);
        let res = a as f64 / b as f64;
        assert_eq!(v1.clone() / v2.clone(), Ok( Float(res) ));

        let a: Value = "hello".into();
        let b: Value = " there".into();
        let c = a + b;
        assert_eq!(c, Ok("hello there".into()));

        let a: Value = "hello".into();
        let b = Float(10.0);
        let c = a / b;
        assert_eq!(c, Err(OperationErrorKind::IncompatibleOperands));

        let a = Float(10.0);
        assert_eq!(-a, Ok( Float(-10.0) ));

        let a: Value = "hello".into();
        let b = -a;
        assert_eq!(b, Err(OperationErrorKind::IllegalNegation));
    }
}
