use std::fmt::{Display,Formatter,Result as FResult};

use token_type::Type;
use literal::Literal;

type Line = u64;
type Column = u64;
type Position = (Line, Column);

type Lexeme = String;

pub struct Token {
    pub token_type: Type,
    pub lexeme: Lexeme,
    pub literal: Option<Literal>,
    pub position: Position,
}

impl Token {
    pub fn new(tt: Type, lex: Lexeme, lit: Option<Literal>, pos: Position) -> Self {
        Token {
            token_type: tt,
            lexeme: lex,
            literal: lit,
            position: pos,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        match self.literal {
            None => write!(f, "<Token type: {:?}, lexeme: {:?}, position: ({}, {})>",
                                self.token_type, self.lexeme, self.position.0, self.position.1),
            Some(ref lit) => write!(f, "<Token type: {:?}, lexeme: {:?}, literal: {:?}, position: ({}, {})>",
                                self.token_type, self.lexeme, lit, self.position.0, self.position.1),
        }
    }
}
