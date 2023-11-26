pub const TEMPLATE : &'static str = r#"
    use std::collections::HashMap;

    #[derive(Eq,Hash,PartialEq,Clone,Debug)]
    pub enum ErrCode {
        {{enums}}
    }

    pub struct Err {
        code_to_data : HashMap<ErrCode,String>,
    }

    impl Err {
        pub fn new() -> Self {
            use ErrCode::*;
            let xx = [
                {{{data}}}
            ];

            Self {
                code_to_data : xx.into(),
            }
        }

        pub fn get(&self, e : ErrCode) -> String {
            self.code_to_data.get(&e).unwrap().to_string()
        }
    }
"#;

