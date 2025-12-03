pub type Input<'a> = &'a [u8];
pub type Result<'a, O> = nom::IResult<Input<'a>, O, nom::error::Error<Input<'a>>>;

#[macro_export]
macro_rules! impl_parse_for_enum {
    ($type: ident, $number_parser: ident) => {
        impl $type {
            pub fn parse(i: parse::Input) -> parse::Result<Self> {
                use nom::{Parser,
                    combinator::map_res,
                    error::{context, ErrorKind},
                    number::complete::$number_parser,
                };
                let parser = map_res($number_parser, |x| Self::try_from(x).map_err(|_| ErrorKind::Alt));
                context(stringify!($type), parser).parse(i)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_parse_for_enumflags {
    ($type: ident, $number_parser: ident) => {
        impl $type {
            pub fn parse(i: parse::Input) -> parse::Result<enumflags2::BitFlags<Self>> {
                use nom::{Parser,
                    combinator::map_res,
                    error::{context, ErrorKind},
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

/*impl Machine {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        use nom::{Parser,
            combinator::map_res,
            error::{context, ErrorKind},
            number::complete::le_u16,
        };
        (context(
            "Machine",
            map_res(le_u16, |x| Self::try_from(x).map_err(|_| ErrorKind::Alt)),
        )).parse(i)
    }
}*/