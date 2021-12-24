use nom_locate::LocatedSpan;
pub type Span<'a> = LocatedSpan<&'a str>;
pub type IResult<'a, O> = nom::IResult<Span<'a>, O>;

