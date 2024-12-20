use crate::error::{Error, NomError};
use nom::branch::alt;
use nom::character::complete::{i64, line_ending, space0, u64};
use nom::combinator::eof;
use nom::error::{FromExternalError, ParseError};
use nom::sequence::delimited;
use nom::{AsChar, Compare, IResult, InputIter, InputLength, InputTake, Parser, Slice};
use nom_supreme::final_parser::{final_parser, ExtractContext, Location};
use nom_supreme::ParserExt;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::num::NonZero;
use std::ops::ControlFlow::{Break, Continue};
use std::ops::{ControlFlow, RangeFrom};

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

struct InfiniteLoopCheck<I: InputLength, O, E: ParseError<I>, P: Parser<I, O, E>> {
    parser: P,
    phantom_data: PhantomData<(I, O, E)>,
}

impl<I: InputLength, O, E: ParseError<I>, P: Parser<I, O, E>> Parser<I, O, E>
    for InfiniteLoopCheck<I, O, E, P>
{
    fn parse(&mut self, input: I) -> IResult<I, O, E> {
        let len = input.input_len();
        self.parser
            .parse(input)
            .and_then(|r| infinite_loop_check(r, len))
    }
}

impl<I: InputLength, O, E: ParseError<I>, P: Parser<I, O, E>> InfiniteLoopCheckTrait<I, O, E, P>
    for P
{
    fn infinite_loop_check(self) -> InfiniteLoopCheck<I, O, E, P> {
        InfiniteLoopCheck {
            parser: self,
            phantom_data: PhantomData,
        }
    }
}

trait InfiniteLoopCheckTrait<I: InputLength, O, E: ParseError<I>, P: Parser<I, O, E>> {
    fn infinite_loop_check(self) -> InfiniteLoopCheck<I, O, E, P>;
}

fn infinite_loop_check<I: InputLength, O, E: ParseError<I>>(
    res: (I, O),
    old_len: usize,
) -> IResult<I, O, E> {
    let (i, o) = res;
    if i.input_len() == old_len {
        Err(nom::Err::Error(E::from_error_kind(
            i,
            nom::error::ErrorKind::Many0,
        )))
    } else {
        Ok((i, o))
    }
}

pub trait FinalParse<I, O, E> {
    fn final_parse(self, input: I) -> Result<O, E>;
    fn partial_parse(self, input: I) -> Result<O, E>;
}

impl<'a, O, P: Parser<&'a str, O, NomError<'a, &'a str>>>
    FinalParse<&'a str, O, NomError<'static, Location>> for P
{
    fn final_parse(self, input: &'a str) -> Result<O, NomError<'static, Location>> {
        final_parser(self)(input)
    }

    fn partial_parse(self, input: &'a str) -> Result<O, NomError<'static, Location>> {
        match self.complete().parse(input) {
            Ok((_, parsed)) => Ok(parsed),
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                Err(err.extract_context(input))
            }
            Err(nom::Err::Incomplete(..)) => {
                unreachable!("Complete combinator should make this impossible")
            }
        }
    }
}

#[allow(dead_code)]
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
        let mut combined_parser = sep.by_ref().precedes(f.by_ref()).infinite_loop_check();
        loop {
            match combined_parser.parse(i.clone()) {
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

#[allow(dead_code)]
pub fn fold_res_many0<I, O, E, E2, F, G, H, R>(
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
    move |mut input: I| {
        let mut acc = init();
        let mut f_infinite_loop_check = f.by_ref().infinite_loop_check();
        loop {
            match exec_once_res(&mut f_infinite_loop_check, &mut g, input.clone(), acc) {
                Continue((n_acc, i1)) => {
                    input = i1;
                    acc = n_acc;
                }
                Break(r) => {
                    acc = r?;
                    break;
                }
            }
        }

        Ok((input, acc))
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
    move |mut input: I| {
        let mut acc = init();
        let mut f_infinite_loop_check = f.by_ref().infinite_loop_check();
        match exec_once_res(&mut f_infinite_loop_check, &mut g, input.clone(), acc) {
            Continue((n_acc, i1)) => {
                input = i1;
                acc = n_acc;
            }
            Break(r) => {
                r?;
                return Err(nom::Err::Error(E::from_error_kind(
                    input,
                    nom::error::ErrorKind::Many1,
                )));
            }
        }

        loop {
            match exec_once_res(&mut f_infinite_loop_check, &mut g, input.clone(), acc) {
                Continue((n_acc, i1)) => {
                    input = i1;
                    acc = n_acc;
                }
                Break(r) => {
                    acc = r?;
                    break;
                }
            }
        }

        Ok((input, acc))
    }
}

#[allow(dead_code)]
pub fn fold_separated_res_many0<I, O, O2, E, E2, F, F2, G, H, R>(
    mut sep: F2,
    mut f: F,
    mut init: H,
    mut g: G,
) -> impl FnMut(I) -> IResult<I, R, E>
where
    I: Clone + InputLength + Debug,
    F2: Parser<I, O2, E>,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> Result<R, (R, Option<I>, nom::Err<E2>)>,
    H: FnMut() -> R,
    E: ParseError<I> + FromExternalError<I, E2>,
{
    move |mut input: I| {
        let mut acc = init();
        let mut f_infinite_loop_check = f.by_ref().infinite_loop_check();
        match exec_once_res(&mut f_infinite_loop_check, &mut g, input.clone(), acc) {
            Continue((n_acc, i1)) => {
                input = i1;
                acc = n_acc;
            }
            Break(r) => return Ok((input, r?)),
        }

        let mut sep_f_infinite_loop_check = sep.by_ref().precedes(f.by_ref()).infinite_loop_check();

        loop {
            match exec_once_res(&mut sep_f_infinite_loop_check, &mut g, input.clone(), acc) {
                Continue((n_acc, i1)) => {
                    input = i1;
                    acc = n_acc;
                }
                Break(r) => {
                    acc = r?;
                    break;
                }
            }
        }

        Ok((input, acc))
    }
}

pub fn fold_separated_res_many1<I, O, O2, E, E2, F, F2, G, H, R>(
    mut sep: F2,
    mut f: F,
    mut init: H,
    mut g: G,
) -> impl FnMut(I) -> IResult<I, R, E>
where
    I: Clone + InputLength + Debug,
    F2: Parser<I, O2, E>,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> Result<R, (R, Option<I>, nom::Err<E2>)>,
    H: FnMut() -> R,
    E: ParseError<I> + FromExternalError<I, E2>,
{
    move |mut input: I| {
        let mut acc = init();
        let mut f_infinite_loop_check = f.by_ref().infinite_loop_check();
        match exec_once_res(&mut f_infinite_loop_check, &mut g, input.clone(), acc) {
            Continue((n_acc, i1)) => {
                input = i1;
                acc = n_acc;
            }
            Break(r) => {
                r?;
                return Err(nom::Err::Error(E::from_error_kind(
                    input,
                    nom::error::ErrorKind::Many1,
                )));
            }
        }

        let mut sep_f_infinite_loop_check = sep.by_ref().precedes(f.by_ref()).infinite_loop_check();

        loop {
            match exec_once_res(&mut sep_f_infinite_loop_check, &mut g, input.clone(), acc) {
                Continue((n_acc, i1)) => {
                    input = i1;
                    acc = n_acc;
                }
                Break(r) => {
                    acc = r?;
                    break;
                }
            }
        }

        Ok((input, acc))
    }
}

pub fn exec_once_res<I, O, E, E2, F, G, R>(
    f: &mut F,
    g: &mut G,
    input: I,
    acc: R,
) -> ControlFlow<Result<R, nom::Err<E>>, (R, I)>
where
    I: Clone + InputLength + Debug,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> Result<R, (R, Option<I>, nom::Err<E2>)>,
    E: ParseError<I> + FromExternalError<I, E2>,
{
    match f.parse(input.clone()) {
        Err(e) => Break(match e {
            nom::Err::Error(_) => Ok(acc),
            e => Err(e),
        }),
        Ok((i1, o)) => match g(acc, o) {
            Ok(n_acc) => Continue((n_acc, i1)),
            Err((n_acc, error_loc, e)) => match e {
                nom::Err::Error(_) => Break(Ok(n_acc)),
                e => Break(Err(e.map(|ee| {
                    E::from_external_error(
                        error_loc.unwrap_or(input.clone()),
                        nom::error::ErrorKind::Many1,
                        ee,
                    )
                }))),
            },
        },
    }
}
