use std::str::FromStr;
use std::string::ToString;
use std::error::Error;
use std::fmt::{Display,Formatter,Result as FResult};

#[derive(Debug)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
}

impl Literal {
    pub fn parse(s: String) -> ParseResult {
        Self::from_str(&*s)
    }
}

type ParseResult = Result<Literal,ParseLiteralErr>;

impl FromStr for Literal {
    type Err = ParseLiteralErr;

    fn from_str(s: &str) -> ParseResult {
        parse_bool(s)
            .or_else(|_| parse_number(s))
            .or_else(|_| parse_string(s))
    }
}

fn parse_bool(s: &str) -> ParseResult {
    bool::from_str(s)
        .map(|b| Literal::Boolean(b))
        .map_err(|e| ParseLiteralErr {
            literal: s.to_owned(),
            message: format!("{}", e),
        })
}

fn parse_string(s: &str) -> ParseResult {
    // shouldn't need to be here....?
    if !(s.starts_with("\"") && s.ends_with("\"")) {
        return Err(ParseLiteralErr {
            literal: s.to_owned(),
            message: "Incorrectly formatted string!".to_owned(),
        })
    }

    s.get(1..s.len() - 1).map(String::from).map(Literal::String).ok_or(ParseLiteralErr {
        literal: s.to_owned(),
        message: "Empty string!".to_owned(),
    })
}

fn parse_number(s: &str) -> ParseResult {
    f64::from_str(s)
        .map(|n| Literal::Number(n))
        .map_err(|e| ParseLiteralErr {
            literal: s.to_owned(),
            message: format!("{}", e),
        })
}

impl ToString for Literal {
    fn to_string(&self) -> String {
        match *self {
            Literal::String(ref s) => s.clone(),
            Literal::Number(n) => n.to_string(),
            Literal::Boolean(b) => b.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ParseLiteralErr {
    literal: String,
    message: String,
}

impl Error for ParseLiteralErr {}

impl Display for ParseLiteralErr {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "Could not parse literal {}: {}", self.literal, self.message)
    }
}
