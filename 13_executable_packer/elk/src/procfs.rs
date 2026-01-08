use core::fmt;

use nom::{
    AsChar, IResult, Parser, branch,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::space0,
    combinator,
    error::ParseError,
    multi,
    sequence::{delimited, preceded, separated_pair, terminated},
};

fn spaced<I, O, E, P>(parser: P) -> impl Parser<I, Output = O, Error = E>
where
    I: nom::Input,
    I::Item: AsChar,
    E: ParseError<I>,
    P: Parser<I, Output = O, Error = E>,
{
    preceded(space0, terminated(parser, space0))
}

fn dec_number(i: &str) -> IResult<&str, u64> {
    let (i, s) = take_while1(|c: char| c.is_ascii_digit()).parse(i)?;
    let num = u64::from_str_radix(s, 10)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Digit)))?;
    Ok((i, num))
}

fn hex_number(i: &str) -> IResult<&str, u64> {
    let (i, s) = take_while1(|c: char| c.is_ascii_hexdigit()).parse(i)?;
    let num = u64::from_str_radix(s, 16)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Digit)))?;
    Ok((i, num))
}

fn hex_addr(i: &str) -> IResult<&str, delf::Addr> {
    let (i, addr) = hex_number(i)?;
    Ok((i, addr.into()))
}

fn hex_addr_range(i: &str) -> IResult<&str, std::ops::Range<delf::Addr>> {
    let (i, (start, end)) = separated_pair(hex_addr, tag("-"), hex_addr).parse(i)?;
    Ok((i, start..end))
}

pub struct Perms {
    pub r: bool,
    pub w: bool,
    pub x: bool,
    pub p: bool,
}

impl fmt::Debug for Perms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bit = |val, display| {
            if val { display } else { "-" }
        };
        write!(
            f,
            "{}{}{}{}",
            bit(self.r, "r"),
            bit(self.w, "w"),
            bit(self.x, "x"),
            bit(self.p, "p")
        )
    }
}

impl Perms {
    fn parse(i: &str) -> IResult<&str, Perms> {
        fn bit(c: &'static str) -> impl Fn(&str) -> IResult<&str, bool> {
            move |i: &str| -> IResult<&str, bool> {
                branch::alt((
                    combinator::value(false, tag("-")),
                    combinator::value(true, tag(c)),
                ))
                .parse(i)
            }
        }
        let (i, (r, w, x, p)) = (bit("r"), bit("w"), bit("x"), bit("p")).parse(i)?;
        Ok((i, Perms { r, w, x, p }))
    }
}

pub struct Dev {
    pub major: u64,
    pub minor: u64,
}

impl fmt::Debug for Dev {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.major, self.minor)
    }
}

impl Dev {
    fn parse(i: &str) -> IResult<&str, Dev> {
        let (i, (major, minor)) = separated_pair(hex_number, tag(":"), hex_number).parse(i)?;
        Ok((i, Dev { major, minor }))
    }
}

#[derive(Debug)]
pub enum Source<'a> {
    Anonymous,
    #[allow(unused)]
    Special(&'a str),
    File(&'a str),
}

impl Source<'_> {
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    fn parse(i: &str) -> IResult<&str, Source<'_>> {
        fn is_path_character(c: char) -> bool {
            // workaround
            c != ']' && !c.is_whitespace()
        }

        fn path(i: &str) -> IResult<&str, &str> {
            take_while(is_path_character).parse(i)
        }

        branch::alt((
            combinator::map(delimited(tag("["), path, tag("]")), Source::Special),
            combinator::map(path, |s| {
                if s.is_empty() {
                    Source::Anonymous
                } else {
                    Source::File(s)
                }
            }),
        ))
        .parse(i)
    }
}

#[derive(Debug)]
pub struct Mapping<'a> {
    pub addr_range: std::ops::Range<delf::Addr>,
    pub perms: Perms,
    pub offset: delf::Addr,
    #[allow(unused)]
    pub dev: Dev,
    #[allow(unused)]
    pub len: u64,
    pub source: Source<'a>,
    pub deleted: bool,
}

impl Mapping<'_> {
    fn parse(i: &str) -> IResult<&str, Mapping<'_>> {
        let (i, (addr_range, perms, offset, dev, len, source, deleted)) = (
            spaced(hex_addr_range),
            spaced(Perms::parse),
            spaced(hex_addr),
            spaced(Dev::parse),
            spaced(dec_number),
            spaced(Source::parse),
            spaced(combinator::map(combinator::opt(tag("(deleted)")), |o| {
                o.is_some()
            })),
        )
            .parse(i)?;
        let res = Mapping {
            addr_range,
            perms,
            offset,
            dev,
            len,
            source,
            deleted,
        };
        Ok((i, res))
    }
}

pub fn mappings(i: &str) -> IResult<&str, Vec<Mapping<'_>>> {
    combinator::all_consuming(multi::many0(terminated(spaced(Mapping::parse), tag("\n")))).parse(i)
}
