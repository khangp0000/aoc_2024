use crate::error::{Error, NomError};
use nom::branch::alt;
use nom::character::complete::{i64, line_ending, space0, u64};
use nom::combinator::eof;
use nom::error::{FromExternalError, ParseError};
use nom::sequence::delimited;
use nom::{AsChar, Compare, IResult, InputIter, InputLength, InputTake, Parser, Slice};
use nom_supreme::final_parser::{final_parser, Location};
use nom_supreme::ParserExt;
use std::fmt::Debug;
use std::num::NonZero;
use std::ops::RangeFrom;

/// A combinator that takes a parser `inner` and produces a parser that also consumes the following
/// line ending or eof, returning the output of `inner`.
pub fn single_line<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E> + 'a,
{
    inner.terminated(alt((line_ending, eof)))
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes the following
/// line ending, returning the output of `inner`.
pub fn single_line_not_eof<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E> + 'a,
{
    inner.terminated(line_ending)
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing space and tab, returning the output of `inner`.
pub fn trim_space<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E> + 'a,
{
    delimited(space0, inner, space0)
}

pub fn ures<I, E: ParseError<I>>(input: I) -> IResult<I, crate::utils::ures, E>
where
    I: InputIter + Slice<RangeFrom<usize>> + InputLength,
    <I as InputIter>::Item: AsChar,
{
    u64.map(|v| v as crate::utils::ures).parse(input)
}

pub fn ires<I, E: ParseError<I>>(input: I) -> IResult<I, crate::utils::ires, E>
where
    I: InputIter + Slice<RangeFrom<usize>> + InputLength + InputTake + Clone,
    <I as InputIter>::Item: AsChar,
    I: for<'a> Compare<&'a [u8]>,
{
    i64.map(|v| v as crate::utils::ires).parse(input)
}

pub fn non_zero_ures(input: &str) -> IResult<&str, NonZero<crate::utils::ures>, NomError<'_>> {
    ures.map_res(|v| NonZero::new(v).ok_or_else(|| Error::ParseError("got 0 for non-zero".into())))
        .context("parse non-zero ures")
        .parse(input)
}

pub trait FinalParse<I, O, E> {
    fn final_parse(self, input: I) -> Result<O, E>;
}

impl<'a, O, P: Parser<&'a str, O, NomError<'a, &'a str>>>
    FinalParse<&'a str, O, NomError<'static, Location>> for P
{
    fn final_parse(self, input: &'a str) -> Result<O, NomError<'static, Location>> {
        final_parser(self)(input)
    }
}

pub fn fold_separated_many0<I, O, O2, E, F, G, H, R, S>(
    mut sep: S,
    mut f: F,
    mut init: H,
    mut g: G,
) -> impl FnMut(I) -> IResult<I, R, E>
where
    I: Clone + InputLength,
    F: Parser<I, O, E>,
    S: Parser<I, O2, E>,
    E: ParseError<I>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
{
    move |mut i: I| {
        let mut res = init();

        match f.parse(i.clone()) {
            Err(nom::Err::Error(_)) => return Ok((i, res)),
            Err(e) => return Err(e),
            Ok((i1, o)) => {
                res = g(res, o);
                i = i1;
            }
        }

        loop {
            let len = i.input_len();
            match sep.parse(i.clone()) {
                Err(nom::Err::Error(_)) => return Ok((i, res)),
                Err(e) => return Err(e),
                Ok((i1, _)) => {
                    // infinite loop check: the parser must always consume
                    if i1.input_len() == len {
                        return Err(nom::Err::Error(E::from_error_kind(
                            i1,
                            nom::error::ErrorKind::Many0,
                        )));
                    }

                    match f.parse(i1.clone()) {
                        Err(nom::Err::Error(_)) => return Ok((i, res)),
                        Err(e) => return Err(e),
                        Ok((i2, o)) => {
                            res = g(res, o);
                            i = i2;
                        }
                    }
                }
            }
        }
    }
}

pub fn fold_res_many1<I, O, E, E2, F, G, H, R>(
    mut f: F,
    mut init: H,
    mut g: G,
) -> impl FnMut(I) -> IResult<I, R, E>
where
    I: Clone + InputLength + Debug,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> Result<R, (R, Option<I>, nom::Err<E2>)>,
    H: FnMut() -> R,
    E: ParseError<I> + FromExternalError<I, E2>,
{
    move |i: I| {
        let _i = i.clone();
        let init = init();

        match f.parse(_i) {
            Err(nom::Err::Error(_)) => Err(nom::Err::Error(E::from_error_kind(
                i,
                nom::error::ErrorKind::Many1,
            ))),
            Err(e) => Err(e),
            Ok((i1, o)) => {
                match g(init, o) {
                    Ok(mut acc) => {
                        let mut input = i1;
                        loop {
                            let _input = input.clone();
                            let len = input.input_len();

                            match f.parse(_input) {
                                Err(nom::Err::Error(_)) => {
                                    break;
                                }
                                Err(e) => return Err(e),
                                Ok((i, o)) => {
                                    // infinite loop check: the parser must always consume
                                    if i.input_len() == len {
                                        return Err(nom::Err::Failure(E::from_error_kind(
                                            i,
                                            nom::error::ErrorKind::Many1,
                                        )));
                                    }

                                    match g(acc, o) {
                                        Ok(n_acc) => {
                                            acc = n_acc;
                                            input = i;
                                        }
                                        Err((n_acc, error_loc, e)) => match e {
                                            nom::Err::Error(_) => {
                                                acc = n_acc;
                                                break;
                                            }
                                            e => {
                                                return Err(e.map(|ee| {
                                                    E::from_external_error(
                                                        error_loc.unwrap_or(input),
                                                        nom::error::ErrorKind::Many1,
                                                        ee,
                                                    )
                                                }))
                                            }
                                        },
                                    };
                                }
                            }
                        }
                        Ok((input, acc))
                    }
                    Err((_, error_loc, e)) => Err(e.map(|ee| {
                        E::from_external_error(
                            error_loc.unwrap_or(i1),
                            nom::error::ErrorKind::Many1,
                            ee,
                        )
                    })),
                }
            }
        }
    }
}
