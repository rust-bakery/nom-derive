use nom::IResult;

pub trait Parse<I, O, E> {
    fn parse(i: I) -> IResult<I, O, E>;
}
