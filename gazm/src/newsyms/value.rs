#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Null,
    Macro,
    Signed(i64),
    Unsigned(u64),
    Text(String),
    Double(f64),
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
            Signed(a) => format!("{a}"),
            Unsigned(a) => format!("{a}"),
            Double(a) => format!("{a}"),
            Text(a) => format!("{a}"),
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
            Unsigned(_) | Signed(_) | Double(_) => true,
            _ => false,
        }
    }

    pub fn as_double(self) -> Self {
        use Value::*;
        match self {
            Signed(a) => Double(a as f64),
            Unsigned(a) => Double(a as f64),
            Double(_) => self,
            _ => Null,
        }
    }

    pub fn as_unsigned(self) -> Self {
        use Value::*;
        match self {
            Signed(a) => Unsigned(a as u64),
            Unsigned(_) => self,
            Double(a) => Unsigned(a as u64),
            _ => Null,
        }
    }

    pub fn as_signed(self) -> Self {
        use Value::*;
        match self {
            Signed(_) => self,
            Unsigned(a) => Signed(a as i64),
            Double(a) => Signed(a as i64),
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
    type Output = Value;

    fn add(self, rhs: Value) -> Self::Output {
        use Value::*;
        match (self, &rhs) {
            (Signed(a), Signed(b)) => Signed(a + b),
            (Signed(a), Unsigned(b)) => Signed(a + *b as i64),
            (Signed(a), Double(b)) => Double(a as f64 + b),
            (Unsigned(a), Signed(b)) => Signed(a as i64 + b),
            (Unsigned(a), Unsigned(b)) => Unsigned(a + b),
            (Unsigned(a), Double(b)) => Double(a as f64 + b),
            (Double(a), Signed(b)) => Double(a + *b as f64),
            (Double(a), Unsigned(b)) => Double(a + *b as f64),
            (Double(a), Double(b)) => Double(a + *b as f64),
            (Text(a), Text(b)) => Text(a + b),
            _ => Null,
        }
    }
}

impl std::ops::Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Value) -> Self::Output {
        use Value::*;
        match (self, &rhs) {
            (Signed(a), Signed(b)) => Signed(a - b),
            (Signed(a), Unsigned(b)) => Signed(a - *b as i64),
            (Signed(a), Double(b)) => Double(a as f64 - b),
            (Unsigned(a), Signed(b)) => Signed(a as i64 - b),
            (Unsigned(a), Unsigned(b)) => Unsigned(a - b),
            (Unsigned(a), Double(b)) => Double(a as f64 - b),
            (Double(a), Signed(b)) => Double(a - *b as f64),
            (Double(a), Unsigned(b)) => Double(a - *b as f64),
            (Double(a), Double(b)) => Double(a - *b as f64),
            _ => Null,
        }
    }
}
impl std::ops::Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Value) -> Self::Output {
        use Value::*;
        match (self, &rhs) {
            (Signed(a), Signed(b)) => Signed(a * b),
            (Signed(a), Unsigned(b)) => Signed(a * *b as i64),
            (Signed(a), Double(b)) => Double(a as f64 * b),
            (Unsigned(a), Signed(b)) => Signed(a as i64 * b),
            (Unsigned(a), Unsigned(b)) => Unsigned(a * b),
            (Unsigned(a), Double(b)) => Double(a as f64 * b),
            (Double(a), Signed(b)) => Double(a * *b as f64),
            (Double(a), Unsigned(b)) => Double(a * *b as f64),
            (Double(a), Double(b)) => Double(a * *b as f64),
            _ => Null,
        }
    }
}

impl std::ops::Div for Value {
    type Output = Value;

    fn div(self, rhs: Value) -> Self::Output {
        use Value::*;
        match (self, &rhs) {
            (Signed(a), Signed(b)) => Signed(a / b),
            (Signed(a), Unsigned(b)) => Signed(a / *b as i64),
            (Signed(a), Double(b)) => Double(a as f64 / b),
            (Unsigned(a), Signed(b)) => Signed(a as i64 / b),
            (Unsigned(a), Unsigned(b)) => Unsigned(a / b),
            (Unsigned(a), Double(b)) => Double(a as f64 / b),
            (Double(a), Signed(b)) => Double(a / *b as f64),
            (Double(a), Unsigned(b)) => Double(a / *b as f64),
            (Double(a), Double(b)) => Double(a / *b as f64),
            _ => Null,
        }
    }
}

impl std::ops::Rem for Value {
    type Output = Value;

    fn rem(self, rhs: Value) -> Self::Output {
        use Value::*;
        match (self, &rhs) {
            (Signed(a), Signed(b)) => Signed(a % b),
            (Signed(a), Unsigned(b)) => Signed(a % *b as i64),
            (Signed(a), Double(b)) => Double(a as f64 % b),
            (Unsigned(a), Signed(b)) => Signed(a as i64 % b),
            (Unsigned(a), Unsigned(b)) => Unsigned(a % b),
            (Unsigned(a), Double(b)) => Double(a as f64 % b),
            (Double(a), Signed(b)) => Double(a % *b as f64),
            (Double(a), Unsigned(b)) => Double(a % *b as f64),
            (Double(a), Double(b)) => Double(a % *b as f64),
            _ => Null,
        }
    }
}

impl std::ops::Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        use Value::*;
        match self {
            Signed(a) => Signed(-a),
            Unsigned(a) => Signed(-(a as i64)),
            Double(a) => Double(-a),
            _ => Null,
        }
    }
}

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
        let v2   = Unsigned(b);
        assert_eq!(v1 * v2, Unsigned(a * b));

        let v1 = Double(a as f64);
        let v2   = Unsigned(b);
        assert_eq!(v2.clone() * v1.clone(), Double(( a * b ) as f64));
        assert_eq!(v1 * v2, Double(( a * b ) as f64));

        let v1 = Double(a as f64);
        let v2   = Unsigned(b);
        let res = a as f64 / b as f64;
        assert_eq!(v1.clone() / v2.clone(), Double(res));

        let a : Value = "hello".into();
        let b : Value = " there".into();
        let c = a + b;
        assert_eq!(c, "hello there".into());

        let a : Value = "hello".into();
        let b = Double(10.0);
        let c = a / b;
        assert_eq!(c, Null);

        let a = Double(10.0);
        assert_eq!(-a, Double(-10.0));

    }
}
