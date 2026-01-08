#[macro_export]
macro_rules! impl_parse_for_enum {
    ($type: ident, $number_parser: ident) => {
        impl $type {
            pub fn parse(full_input: parse::Input) -> parse::Result<Self> {
                use nom::number::complete::$number_parser;
                let (i, val) = $number_parser(full_input)?;
                match Self::try_from(val) {
                    Ok(val) => Ok((i, val)),
                    Err(_) => Err(nom::Err::Failure(parse::Error::from_string(
                        full_input,
                        format!("Unknown {} {} (0x{:x})", stringify!($type), val, val),
                    ))),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_parse_for_bitenum {
    ($type: ident, $bits: expr) => {
        impl $type {
            pub fn parse(full_input: parse::BitInput) -> parse::BitResult<Self> {
                use nom::bits::complete::take;
                let (i, val): (_, u8) = take($bits)(full_input)?;
                match Self::try_from(val) {
                    Ok(val) => Ok((i, val)),
                    Err(_) => Err(nom::Err::Failure(parse::Error::from_string(
                        full_input,
                        format!("Unknown {} {} (0x{:x})", stringify!($type), val, val),
                    ))),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_parse_for_enumflags {
    ($type: ident, $number_parser: ident) => {
        impl $type {
            pub fn parse(i: parse::Input) -> parse::Result<enumflags2::BitFlags<Self>> {
                use nom::{
                    Parser,
                    combinator::map_res,
                    error::{ErrorKind, context},
                    number::complete::$number_parser,
                };
                let parser = map_res($number_parser, |x| {
                    enumflags2::BitFlags::<Self>::from_bits(x).map_err(|_| ErrorKind::Alt)
                });
                context(stringify!($type), parser).parse(i)
            }
        }
    };
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Nom(nom::error::ErrorKind),
    Context(&'static str),
    String(String),
}

pub struct Error<I> {
    pub errors: Vec<(I, ErrorKind)>,
}

impl<I> nom::error::ParseError<I> for Error<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        let errors = vec![(input, ErrorKind::Nom(kind))];
        Self { errors }
    }

    fn append(input: I, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.errors.push((input, ErrorKind::Nom(kind)));
        other
    }
}

pub type Input<'a> = &'a [u8];
pub type Result<'a, O> = nom::IResult<Input<'a>, O, Error<Input<'a>>>; //nom::error::

pub type BitInput<'a> = (&'a [u8], usize);
pub type BitResult<'a, O> = nom::IResult<BitInput<'a>, O, Error<BitInput<'a>>>; //nom::error::

impl<'a> nom::ErrorConvert<Error<&'a [u8]>> for Error<(&'a [u8], usize)> {
    fn convert(self) -> Error<&'a [u8]> {
        let errors = self
            .errors
            .into_iter()
            .map(|((rest, offset), err)| (&rest[offset / 8..], err))
            .collect();
        Error { errors }
    }
}

impl<I> nom::error::ContextError<I> for Error<I> {
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other.errors.push((input, ErrorKind::Context(ctx)));
        other
    }
}

impl<I> nom::error::FromExternalError<I, nom::error::ErrorKind> for Error<I> {
    fn from_external_error(
        input: I,
        // silent kind
        _kind: nom::error::ErrorKind,
        e: nom::error::ErrorKind,
    ) -> Self {
        let errors = vec![(input, ErrorKind::Nom(e))]; //, (input, e)
        Error { errors }
    }
}

impl std::fmt::Debug for Error<&[u8]> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (input, err) in &self.errors {
            writeln!(f, "{err:?}")?;
            writeln!(f, "input: {:?}", crate::HexDump(input))?;
        }
        Ok(())
    }
}

impl<I> Error<I> {
    pub fn from_string<S: Into<String>>(input: I, s: S) -> Self {
        let errors = vec![(input, ErrorKind::String(s.into()))];
        Self { errors }
    }
}
