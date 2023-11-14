////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ByteSizes {
    Zero,
    Bits5(i8),
    Byte(i8),
    Word(i16),
}

#[allow(dead_code)]
impl ByteSizes {
    pub fn promote(&mut self) {
        *self = match self {
            Self::Zero => Self::Zero,
            Self::Bits5(v) => Self::Byte(*v),
            Self::Byte(v) => Self::Word(*v as i16),
            Self::Word(v) => Self::Word(*v),
        };
    }
}

pub trait ByteSize {
    fn byte_size(&self) -> ByteSizes;
}

impl ByteSize for i64 {
    fn byte_size(&self) -> ByteSizes {
        let v = *self;
        if v == 0 {
            ByteSizes::Zero
        } else if v > -16 && v < 16 {
            ByteSizes::Bits5(v as i8)
        } else if v > -128 && v < 128 {
            ByteSizes::Byte(v as i8)
        } else {
            ByteSizes::Word(v as i16)
        }
    }
}
