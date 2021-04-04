use nom::bytes::streaming::take;
use nom::combinator::{complete, map_res, opt};
use nom::error::{Error, FromExternalError, ParseError};
use nom::multi::{many0, many_m_n};
use nom::number::streaming::*;
use nom::sequence::pair;
use nom::*;
use std::convert::TryFrom;
use std::ops::RangeFrom;

/// Common trait for all parsers in nom-derive
pub trait Parse<I, E = Error<I>>
where
    E: ParseError<I>,
    Self: Sized,
{
    /// Parse input, redardless of endianness
    fn parse(i: I) -> IResult<I, Self, E>;

    /// Parse input as Big-Endian
    fn parse_be(i: I) -> IResult<I, Self, E> {
        Self::parse(i)
    }

    /// Parse input as Little-Endian
    fn parse_le(i: I) -> IResult<I, Self, E> {
        Self::parse(i)
    }
}

macro_rules! impl_primitive_type {
    ( $ty:ty, $be_fn: ident, $le_fn: ident ) => {
        impl<I, E> Parse<I, E> for $ty
        where
            E: ParseError<I>,
            I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
        {
            fn parse(i: I) -> IResult<I, Self, E> {
                Self::parse_be(i)
            }
            fn parse_be(i: I) -> IResult<I, Self, E> {
                $be_fn(i)
            }
            fn parse_le(i: I) -> IResult<I, Self, E> {
                $le_fn(i)
            }
        }
    };
}

impl_primitive_type!(i8, be_i8, le_i8);
impl_primitive_type!(i16, be_i16, le_i16);
impl_primitive_type!(i32, be_i32, le_i32);
impl_primitive_type!(i64, be_i64, le_i64);
impl_primitive_type!(i128, be_i128, le_i128);

impl_primitive_type!(u8, be_u8, le_u8);
impl_primitive_type!(u16, be_u16, le_u16);
impl_primitive_type!(u32, be_u32, le_u32);
impl_primitive_type!(u64, be_u64, le_u64);
impl_primitive_type!(u128, be_u128, le_u128);

impl_primitive_type!(f32, be_f32, le_f32);
impl_primitive_type!(f64, be_f64, le_f64);

impl<'a, E> Parse<&'a [u8], E> for String
where
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], std::str::Utf8Error>,
{
    fn parse(i: &'a [u8]) -> IResult<&'a [u8], Self, E> {
        let (rem, sz) = <u32>::parse(i)?;
        let (rem, s) = map_res(take(sz as usize), std::str::from_utf8)(rem)?;
        Ok((rem, s.to_owned()))
    }
}

impl<T, I, E> Parse<I, E> for Option<T>
where
    I: Clone,
    E: ParseError<I>,
    T: Parse<I, E>,
{
    fn parse(i: I) -> IResult<I, Self, E> {
        opt(complete(<T>::parse))(i)
    }
    fn parse_be(i: I) -> IResult<I, Self, E> {
        opt(complete(<T>::parse_be))(i)
    }
    fn parse_le(i: I) -> IResult<I, Self, E> {
        opt(complete(<T>::parse_le))(i)
    }
}

impl<T, I, E> Parse<I, E> for Vec<T>
where
    I: Clone + PartialEq,
    E: ParseError<I>,
    T: Parse<I, E>,
{
    fn parse(i: I) -> IResult<I, Self, E> {
        many0(complete(<T>::parse))(i)
    }
    fn parse_be(i: I) -> IResult<I, Self, E> {
        many0(complete(<T>::parse_be))(i)
    }
    fn parse_le(i: I) -> IResult<I, Self, E> {
        many0(complete(<T>::parse_le))(i)
    }
}

impl<T1, T2, I, E> Parse<I, E> for (T1, T2)
where
    I: Clone + PartialEq,
    E: ParseError<I>,
    T1: Parse<I, E>,
    T2: Parse<I, E>,
{
    fn parse(i: I) -> IResult<I, Self, E> {
        pair(T1::parse, T2::parse)(i)
    }
    fn parse_be(i: I) -> IResult<I, Self, E> {
        pair(T1::parse_be, T2::parse_be)(i)
    }
    fn parse_le(i: I) -> IResult<I, Self, E> {
        pair(T1::parse_le, T2::parse_le)(i)
    }
}

/// *Note: this implementation uses const generics and requires rust >= 1.51*
#[rustversion::since(1.51)]
impl<T, I, E, const N: usize> Parse<I, E> for [T; N]
where
    I: Clone + PartialEq,
    E: ParseError<I> + FromExternalError<I, Vec<T>>,
    T: Parse<I, E>,
{
    fn parse(i: I) -> IResult<I, Self, E> {
        map_res(many_m_n(N, N, complete(<T>::parse)), Self::try_from)(i)
    }
    fn parse_be(i: I) -> IResult<I, Self, E> {
        map_res(many_m_n(N, N, complete(<T>::parse_be)), |v| {
            Self::try_from(v)
        })(i)
    }
    fn parse_le(i: I) -> IResult<I, Self, E> {
        map_res(many_m_n(N, N, complete(<T>::parse_le)), |v| {
            Self::try_from(v)
        })(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_trait_vec() {
        let input: &[u8] = b"\x00\x01\x02\x03";

        type T = Vec<u8>;
        let res: IResult<_, _, Error<&[u8]>> = <T>::parse(input);
        assert_eq!(res.unwrap(), (b"" as &[u8], vec![0, 1, 2, 3]));
    }

    #[test]
    fn test_parse_trait_array() {
        let input: &[u8] = b"\x00\x01\x02\x03";

        type T = [u8; 4];
        let res: IResult<_, _, Error<&[u8]>> = <T>::parse(input);
        assert_eq!(res.unwrap(), (b"" as &[u8], [0, 1, 2, 3]));
    }

    #[test]
    fn test_parse_trait_string() {
        let input: &[u8] = b"\x00\x00\x00\x04abcd";

        type T = String;
        let res: IResult<_, _, Error<&[u8]>> = <T>::parse_le(input);
        assert_eq!(res.unwrap(), (b"" as &[u8], String::from("abcd")));
    }
}
