use thin_vec::ThinVec;

////////////////////////////////////////////////////////////////////////////////
#[derive(Debug,Clone)]
pub struct ScopedName<'a> {
    input: &'a str,
    symbol: &'a str,
    path: ThinVec<&'a str>,
    absolute: bool,
}

impl<'a> std::fmt::Display for ScopedName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.input) 
    }
}
impl<'a> ScopedName<'a> {
    pub fn is_abs(&self) -> bool {
        self.absolute
    }

    pub fn is_relative(&self) -> bool {
        !self.is_abs()
    }

    pub fn symbol(&self) -> &str {
        self.symbol
    }

    pub fn path(&self) -> &[&str] {
        &self.path
    }

    pub fn path_as_string(&self) -> String {
        let path = self.path.join("::");
        if self.is_abs() {
            format!("::{path}")

        } else {
            path
        }
    }

    pub fn new(input: &'a str) -> Self {
        let splits : ThinVec<_> = input.split("::").collect();
        let len = splits.len();

        let (path,symbol) = match len {
            0 => panic!(),
            _ => (&splits[0..len-1],splits[len-1] ),
        };

        let absolute = !path.is_empty() && path[0].is_empty();

        let path : ThinVec<_> = if absolute {
            path[1..].into_iter().map(|p| *p).collect()
        } else {
            path.into_iter().map(|p| *p).collect()
        };

        Self {
            input,
            symbol,
            path,
            absolute
        }
    }
}

