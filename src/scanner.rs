use std::iter;
use std::str::{FromStr,Chars};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::error::Error;
use std::collections::{HashMap};

use token::Token;
use token_type::Type as TT;
use literal::Literal as Lit;

pub fn scan(source: String) -> Result<Tokens, ScanError> {
    Scanner::new(source.chars().peekable()).collect()
}

type Tokens = Vec<Token>;

type Line = u64;
type Column = u64;
type Position = (Line, Column);

#[derive(Debug)]
pub struct ScanError {
    position: Position,
    message: String,
}

impl Error for ScanError {}

impl Display for ScanError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Error reading code at line {}, column {}: {}",
               self.position.0, self.position.1, self.message)
    }
}

type ScanSource<'a> = iter::Peekable<Chars<'a>>;

struct Scanner<'a> {
    source: ScanSource<'a>,
    position: Position,
    current: String,
    reserved_words: HashMap<&'a str, TT>,
}

impl<'a> Scanner<'a> {
    fn new(chars: ScanSource<'a>) -> Self {
        Scanner {
            source: chars,
            position: (0, 0),
            current: String::new(),
            reserved_words: reserved_words(),
        }
    }

    fn forward(&mut self) {
        self.position.1 += 1;
    }

    fn down(&mut self) {
        self.position = (self.position.0 + 1, 0);
    }

    fn skip_forward(&mut self) -> Option<Scan> {
        self.forward();
        self.next()
    }

    fn skip_down(&mut self) -> Option<Scan> {
        self.down();
        self.next()
    }

    fn consume(&mut self, ch: char) {
        if ch == '\n' {
            self.down();
        } else {
            self.forward();
        }
        self.current.push(ch);
    }

    fn token(&self, tt: TT) -> Token {
        let lexeme = self.current.trim();
        // the clones here make me think i should bite the bullet
        // and add lifetimes and make current a &mut str...
        match Lit::from_str(lexeme) {
            Ok(lit) => Token::new(tt, lexeme.to_owned(), Some(lit), self.position),
            Err(_) => Token::new(tt, lexeme.to_owned(), None, self.position),
        }
    }

    fn emit(&mut self, tt: TT) -> Token {
        let tok = self.token(tt);
        self.current = String::new();
        tok
    }

    fn unexpected_error(&self) -> ScanError {
        ScanError {
            position: self.position,
            message: format!("Unexpected character: {:?}", self.current),
        }
    }

    fn taste(&mut self, mc: char) -> Option<char> {
        self.source.peek()
            .and_then(|&c| if c == mc {
                Some(c)
            } else {
                None
            })
            .and_then(|_| self.source.next().map(|c| c))
    }

    fn digest(&mut self, mc: char, emission: TT) -> Token {
        self.consume(mc);
        self.emit(emission)
    }

    fn skip_til(&mut self, stop: char) {
        loop {
            if self.source.next().map(|c| c == stop).unwrap_or(true) {
                break
            } else {
                self.forward();
            }
        }
    }

    fn skip_line(&mut self) -> Option<Scan> {
        self.skip_til('\n');
        self.skip_down()
    }

    fn slurp_til(&mut self, stop: &Fn(char) -> bool) {
        loop {
            if self.source.peek().map(|&c| stop(c)).unwrap_or(true) {
                break
            } else {
                match self.source.next() {
                    None => break,
                    Some(c) => self.consume(c),
                }
            }
        }
    }

    fn slurp_while(&mut self, keep_going: &Fn(char) -> bool) {
        loop {
            if self.source.peek().map(|&c| !keep_going(c)).unwrap_or(false) {
                break
            } else {
                match self.source.next() {
                    None => break,
                    Some(c) => self.consume(c),
                }
            }
        }
    }

    fn identifier(&mut self, ch: char) -> Token {
        self.consume(ch);
        self.slurp_while(&is_alphanumeric);
        let tt = self.reserved_words
            .get(&*self.current)
            .map(|&tt| tt)
            .unwrap_or(TT::Identifier);
        self.emit(tt)
    }

    fn number(&mut self, ch: char) -> Token {
        self.consume(ch);
        self.slurp_while(&is_digit);
        match self.source.clone().take(2).collect::<Vec<char>>().get(0..2) {
            Some(&[c1, c2]) => if is_dot(c1) && is_digit(c2) {
                let dot = self.source.next().unwrap();
                self.consume(dot);
                self.slurp_while(&is_digit);
            },
            _ => (),
        };
        self.emit(TT::Number)
    }
}

type Scan = Result<Token, ScanError>;

impl<'a> iter::Iterator for Scanner<'a> {
    type Item = Scan;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.next().and_then(|ch| {
            // this feels like maybe there could be more complex matching
            // maybe some sort of "scan instruction" type?
            match ch {
                '(' => some_ok(self.digest(ch, TT::LeftParen)),
                ')' => some_ok(self.digest(ch, TT::RightParen)),
                '{' => some_ok(self.digest(ch, TT::LeftBrace)),
                '}' => some_ok(self.digest(ch, TT::RightBrace)),
                ',' => some_ok(self.digest(ch, TT::Comma)),
                '.' => some_ok(self.digest(ch, TT::Dot)),
                '-' => some_ok(self.digest(ch, TT::Minus)),
                '+' => some_ok(self.digest(ch, TT::Plus)),
                ';' => some_ok(self.digest(ch, TT::Semicolon)),
                '*' => some_ok(self.digest(ch, TT::Star)),
                '!' => {
                    self.consume(ch);
                    self.taste('=')
                        .and_then(|nc| some_ok(self.digest(nc, TT::BangEqual)))
                        .or_else(|| some_ok(self.emit(TT::Bang)))
                },
                '=' => {
                    self.consume(ch);
                    self.taste('=')
                        .and_then(|nc| some_ok(self.digest(nc, TT::EqualEqual)))
                        .or_else(|| some_ok(self.emit(TT::Equal)))
                },
                '<' => {
                    self.consume(ch);
                    self.taste('=')
                        .and_then(|nc| some_ok(self.digest(nc, TT::LessEqual)))
                        .or_else(|| some_ok(self.emit(TT::Less)))
                },
                '>' => {
                    self.consume(ch);
                    self.taste('=')
                        .and_then(|nc| some_ok(self.digest(nc, TT::GreaterEqual)))
                        .or_else(|| some_ok(self.emit(TT::Greater)))
                },
                '/' => {
                    match self.taste('/') {
                        Some(_) => self.skip_line(),
                        None => some_ok(self.digest(ch, TT::Slash)),
                    }
                },

                // strings
                '"' => {
                    self.consume(ch);
                    self.slurp_til(&|c| c == '"');
                    match self.source.next() {
                        None => some_err(self.unexpected_error()),
                        Some(c) => some_ok(self.digest(c, TT::String)),
                    }
                }

                // whitespace
                ' ' => self.skip_forward(),
                '\r' => self.skip_forward(),
                '\t' => self.skip_forward(),
                '\n' => self.skip_down(),
                _ => if is_digit(ch) {
                    some_ok(self.number(ch))
                } else if is_alpha(ch) {
                    some_ok(self.identifier(ch))
                } else {
                    some_err(self.unexpected_error())
                }
            }
        })
    }
}

fn is_alphanumeric(ch: char) -> bool {
    is_alpha(ch) || is_digit(ch)
}

fn is_alpha(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

fn is_digit(ch: char) -> bool {
    ch.is_digit(10)
}

fn is_dot(ch: char) -> bool {
    ch == '.'
}

fn some_ok<T, E>(x: T) -> Option<Result<T,E>> {
    Some(Ok(x))
}

fn some_err<T,E>(x: E) -> Option<Result<T,E>> {
    Some(Err(x))
}

// in reality this should probably use lazy_static! or phf
fn reserved_words<'a>() -> HashMap<&'a str, TT> {
    let mut rs = HashMap::new();
    rs.insert("and",    TT::And);
    rs.insert("class",  TT::Class);
    rs.insert("else",   TT::Else);
    rs.insert("false",  TT::False);
    rs.insert("for",    TT::For);
    rs.insert("fun",    TT::Fun);
    rs.insert("if",     TT::If);
    rs.insert("nil",    TT::Nil);
    rs.insert("or",     TT::Or);
    rs.insert("print",  TT::Print);
    rs.insert("return", TT::Return);
    rs.insert("super",  TT::Super);
    rs.insert("this",   TT::This);
    rs.insert("true",   TT::True);
    rs.insert("var",    TT::Var);
    rs.insert("while",  TT::While);
    rs
}
