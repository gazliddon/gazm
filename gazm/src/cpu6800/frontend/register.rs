use emu6800::cpu_core::RegEnum;
use std::str::FromStr;

use crate::{
    frontend::{err_error, error, get_label_string, PResult, TSpan},
    help::ErrCode::ExpectedRegister6809,
};

use unraveler::match_span as ms;

pub fn parse_this_reg_local(input: TSpan, r: RegEnum) -> PResult<RegEnum> {
    use crate::help::ErrCode;

    let (rest, (sp, matched)) = ms(get_register)(input)?;

    if matched != r {
        err_error(sp, ErrCode::ExpectedRegister6809)
    } else {
        Ok((rest, matched))
    }
}

pub fn get_this_reg(r: RegEnum) -> impl FnMut(TSpan) -> PResult<RegEnum> + Copy {
    move |i| parse_this_reg_local(i, r)
}

/// Parse a single register
pub fn get_register(input: TSpan) -> PResult<RegEnum> {
    let (rest, (sp, text)) = ms(get_label_string)(input)?;

    text.as_str()
        .parse::<RegEnum>()
        .map(|reg| (rest, reg))
        .map_err(|_| error(sp, ExpectedRegister6809))
}
