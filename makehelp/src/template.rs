pub const TEMPLATE : &'static str = r#"
    use std::collections::HashMap;

    pub enum ErrCode {
        {{enums}}
    }

    pub struct Err {
        code_to_data : HashMap<ErrCode,String>,
    }

    impl Err {
        pub fn new() -> Self {
            let xx = [
                {{{data}}}
            ];

            Self {
                code_to_data : xx.into(),
            }
        }

        pub fn get(&self, e : Err) -> String {
            self.code_to_data.get(&e).unwrap()
        }
    }
"#;

