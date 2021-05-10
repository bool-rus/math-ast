use std::str::FromStr;
use std::fmt::{Debug, Formatter};
use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};
use parser::faces::Function;
use super::num::Num;
use super::num::Integer;
use super::num::Float;
use parser::faces::FnFunction;

#[derive(Debug)]
pub enum Lexem {
    Letter(String),
    Op(Operand),
    Open,
    Close,
    Comma,
}


impl Lexem {
    fn special(ch: char) -> Option<Lexem> {
        match ch {
            '(' => Some(Lexem::Open),
            ')' => Some(Lexem::Close),
            ',' => Some(Lexem::Comma),
            _ => Some(Lexem::Op(Operand::from(ch)?))
        }
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Eq, PartialEq)]
pub enum Operand {
    Low(char),
    High(char),
    Highest(char),
}

impl ToString for Operand {
    fn to_string(&self) -> String {
        match self {
            Operand::Low(ch) => ch,
            Operand::High(ch) => ch,
            Operand::Highest(ch) => ch,
        }.to_string()
    }
}

impl Operand {
    #[cfg(test)]
    pub fn make_fn<T:'static + Float + Sized>(ch: char) -> Box<Function<T>> {
        Self::from(ch).unwrap().into()
    }
    pub fn more(&self, rhs: &Operand) -> bool {
        match (self, rhs) {
            (Operand::High(..), Operand::Low(..)) => true,
            (Operand::Highest(..), Operand::High(..)) => true,
            (Operand::Highest(..), Operand::Low(..)) => true,
            _ => false,
        }
    }
    fn from(ch: char) -> Option<Operand> {
        match ch {
            '+' | '-' => Some(Operand::Low(ch)),
            '*' | '/' => Some(Operand::High(ch)),
            '^' => Some(Operand::Highest(ch)),
            _ => None,
        }
    }
    fn ch(self) -> char {
        match self {
            Operand::Low(c) => c,
            Operand::High(c) => c,
            Operand::Highest(c) => c,
        }
    }
}

impl Operand {
    fn add<T:Float>(args: Vec<T>) -> Option<T> {
        Some(args[0]+args[1])
    }
    fn sub<T:Float>(args: Vec<T>) -> Option<T> {
        Some(args[0]-args[1])
    }
    fn mul<T:Float>(args: Vec<T>) -> Option<T> {
        Some(args[0]*args[1])
    }
    fn div<T:Float>(args: Vec<T>) -> Option<T> {
        Some(args[0]/args[1])
    }
    fn pow<T:Float>(args: Vec<T>) -> Option<T> {
        Some(args[0].powf(args[1]))
    }
}

impl<T:'static + Float + Sized> Into<Box<Function<T>>> for Operand {
    fn into(self) -> Box<Function<T>> {
        match self.ch() {
            ch @ '+' => FnFunction::new(ch, &Self::add),
            ch @'-' => FnFunction::new(ch, &Self::sub),
            ch @ '*' => FnFunction::new(ch, &Self::mul),
            ch @ '/' => FnFunction::new(ch, &Self::div),
            ch @ '^' => FnFunction::new(ch, &Self::pow),
            _ => unreachable!(),
        }
    }
}


#[derive(Debug)]
enum State {
    None,
    Letter,
    Special,
}

impl From<char> for State {
    fn from(ch: char) -> Self {
        match ch {
            '0' | '.' | '1'...'9' | 'a'...'z' | 'A'...'Z' => State::Letter,
            ch if !Lexem::special(ch).is_none()  => State::Special,
            _ => State::None
        }
    }
}

struct Parser {
    state: State,
    lexemes: Vec<Lexem>,
    buf: String,
}


impl Parser {
    pub fn new() -> Parser {
        Parser {
            state: State::None,
            lexemes: Vec::new(),
            buf: String::new(),
        }
    }
    pub fn process(mut self, ch: char) -> Self {
        let new_state = State::from(ch);
        match (self.state, new_state) {
            (prev, State::None) => self.state = prev,
            (_, s @ State::Letter) => {
                self.state = s;
                self.buf.push(ch);
            }
            (State::Letter, State::Special) => {
                self.state = State::Special;
                let letter = self.buf;
                self.buf = String::new();
                self.end_letter(letter);
                self.lexemes.push(Lexem::special(ch).unwrap());
            }
            (_, State::Special) => {
                self.state = State::Special;
                self.lexemes.push(Lexem::special(ch).unwrap());
            }
        };
        self
    }
    fn end_letter(&mut self, letter: String) {
        if letter.is_empty() {
            return;
        }
        self.lexemes.push(Lexem::Letter(letter));
    }
    pub fn end(mut self) -> Self {
        let letter = self.buf;
        self.buf = String::new();
        self.end_letter(letter);
        self
    }
}

pub fn parse(input: &str) -> Vec<Lexem> {
    let p = input.chars().fold(Parser::new(), |p, ch| p.process(ch)).end();
    p.lexemes
}




#[test]
fn test_parse() {
    let v = parse("y8 - x + y/(8*6.38 - 5)-5x+y8");
    v.iter().fold(0, |s, l| {
        println!("{:?}", l);
        s + 1
    });
}


