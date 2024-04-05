use std::iter::Peekable;
use std::num::ParseIntError;
use std::str::Chars;

use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
enum ParserError {
    UnexpectedSymbol(char),
    UnexpectedEof,
    UnexpectedToken(Token),
    InvalidInt32,
}

impl Error for ParserError {}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> { 
        match self {
            Self::UnexpectedSymbol(c) => write!(f, "Unexpected symbol {c}"),
            Self::UnexpectedEof => write!(f, "Unexpected EOF"),
            Self::UnexpectedToken(token) => write!(f, "Unexpected token: {token:?}"),
            Self::InvalidInt32 => write!(f, "Invalid 32 bit integer"),
        }
    }
}

impl From<ParseIntError> for ParserError {
    fn from(_: ParseIntError) -> Self {
        Self::InvalidInt32
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Op {
    Add,
    Sub,
}

#[derive(Debug, PartialEq)]
enum Token {
    Number(String),
    Identifier(String),
    Operator(Op),
}

#[derive(Debug, PartialEq)]
enum Expression {
    Var(String),
    Number(u32),
    Operation(Box<Expression>, Op, Box<Expression>),
}

fn tokenize(input: &str) -> Tokenizer {
    return Tokenizer(input.chars().peekable());
}

struct Tokenizer<'a>(Peekable<Chars<'a>>);

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token, ParserError>;

    fn next(&mut self) -> Option<Result<Token, ParserError>> {
        let c = self.0.next()?;
        Some(Ok(match c {
            '0'..='9' => {
                let mut num = String::from(c);
                while let Some(c @ '0'..='9') = self.0.peek() {
                    num.push(*c);
                    self.0.next();
                }
                Token::Number(num)
            }
            'a'..='z' => {
                let mut ident = String::from(c);
                while let Some(c @ ('a'..='z' | '_' | '0'..='9')) = self.0.peek() {
                    ident.push(*c);
                    self.0.next();
                }
                Token::Identifier(ident)
            }
            '+' => Token::Operator(Op::Add),
            '-' => Token::Operator(Op::Sub),
            _ => return Some(Err(ParserError::UnexpectedSymbol(c))),
        }))
    }
}

fn parse(input: &str) -> Result<Expression, ParserError> {
    let mut tokens = tokenize(input);

    fn parse_expr<'a>(tokens: &mut Tokenizer<'a>) -> Result<Expression, ParserError> {
        let tok = tokens.next().ok_or(ParserError::UnexpectedEof)??;
        let expr = match tok {
            Token::Number(num) => Expression::Number(num.parse()?),
            Token::Identifier(ident) => Expression::Var(ident),
            Token::Operator(_) => return Err(ParserError::UnexpectedToken(tok)),
        };
        Ok(match tokens.next() {
            None => expr,
            Some(Ok(Token::Operator(op))) => Expression::Operation(
                Box::new(expr),
                op,
                Box::new(parse_expr(tokens)?),
            ),
            Some(Err(e)) => return Err(e.into()),
            Some(Ok(tok)) => return Err(ParserError::UnexpectedToken(tok)),
        })
    }

    parse_expr(&mut tokens)
}

fn main() {
    println!("{:?}", parse("10+foo+20-30-"));
    println!("{:?}", parse("10+foo+20*30"));
    println!("{:?}", parse("10s+foo+20*30"));
    println!("{:?}", parse("10+foo+20-30"));
}