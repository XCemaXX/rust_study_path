use alloc::borrow::Cow;
use core::fmt::Display;
use encore::prelude::*;

extern crate alloc;

#[derive(Clone)]
pub struct Error {
    program_name: &'static str,
    message: Cow<'static, str>,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Error: {}", self.message)?;
        writeln!(f, "Usage: {} input -o output", self.program_name)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Args {
    /// The executable to compress
    pub input: &'static str,
    /// Where to write the compressed executable on disk
    pub output: &'static str,
}

impl Args {
    pub fn parse(env: &Env) -> Self {
        match Self::parse_inner(env) {
            Err(e) => {
                println!("{e}");
                unsafe {
                    syscall::exit(1);
                }
            }
            Ok(x) => x,
        }
    }

    fn parse_inner(env: &Env) -> Result<Self, Error> {
        let mut args = env.args.iter().copied();
        let program_name = args.next().unwrap();
        let mut raw: ArgsRaw = Default::default();

        let err = |message| Error {
            program_name,
            message,
        };

        while let Some(arg) = args.next() {
            if arg.starts_with('-') {
                Self::parse_flag(arg, &mut args, &mut raw, &err)?;
                continue;
            }

            if raw.input.is_some() {
                return Err(err("Multiple input files specified".into()));
            } else {
                raw.input = Some(arg)
            }
        }

        Ok(Args {
            input: raw.input.ok_or_else(|| err("Missing input".into()))?,
            output: raw.output.ok_or_else(|| err("Missing output".into()))?,
        })
    }

    fn parse_flag(
        flag: &'static str,
        args: &mut dyn Iterator<Item = &'static str>,
        raw: &mut ArgsRaw,
        err: &dyn Fn(Cow<'static, str>) -> Error,
    ) -> Result<(), Error> {
        match flag {
            "-o" | "--output" => {
                let output = args
                    .next()
                    .ok_or_else(|| err("Missing output filename after -o / --output".into()))?;
                if raw.output.is_some() {
                    return Err(err("Multiple output files specified".into()));
                }
                raw.output = Some(output);
                Ok(())
            }
            x => Err(err(format!("Unknown flag {x}").into())),
        }
    }
}

#[derive(Default)]
pub struct ArgsRaw {
    pub input: Option<&'static str>,
    pub output: Option<&'static str>,
}
