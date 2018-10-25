use std::str::FromStr;
use std::fmt::{Debug, Formatter};
use std::fmt;
use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};
use parser::faces::Fun;
use super::num::Num;
use super::num::Integer;
use super::num::Float;

#[derive(Debug)]
pub enum Lexem<T> {
    Number(T),
    Letter(String),
    Op(Operand),
    Open,
    Close,
}


impl<T> Lexem<T> {
    fn special(ch: char) -> Option<Lexem<T>> {
        match ch {
            '(' => Some(Lexem::Open),
            ')' => Some(Lexem::Close),
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


impl Operand {
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
    pub fn to_fn<T>(self) -> Fun<T>
        where //X: From<Fn(T,T)->T>,
            T: Float
    {
        match self.ch() {
            '+' => Fun::from(|a, b| a + b),
            '-' => Fun::from(|a, b| a - b),
            '*' => Fun::from(|a, b| a * b),
            '/' => Fun::from(|a, b| a / b),
            '^' => Fun::from(|a:T,b:T |a.powf(b)),
            _ => unreachable!(),
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
            ch if !Lexem::<u8>::special(ch).is_none()  => State::Special,
            _ => State::None
        }
    }
}

impl State {
    fn is_none(&self) -> bool {
        match self {
            State::None => true,
            _ => false,
        }
    }
}

struct Parser<T> {
    state: State,
    lexemes: Vec<Lexem<T>>,
    buf: String,
}


impl<T> Parser<T> where T: Float {
    pub fn new() -> Parser<T> {
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
        match <T>::from_str_radix(&letter, 10) {
            Ok(n) => self.lexemes.push(Lexem::Number(n)),
            Err(_) => self.lexemes.push(Lexem::Letter(letter)),
        }
    }
    pub fn end(mut self) -> Self {
        let letter = self.buf;
        self.buf = String::new();
        self.end_letter(letter);
        self
    }
}

pub fn parse<T>(input: String) -> Vec<Lexem<T>> where T: Float {
    let p = input.chars().fold(Parser::new(), |p, ch| p.process(ch)).end();
    p.lexemes
}


#[cfg(test)]
pub fn make_operand<T>(ch: char) -> Lexem<T> {
    Lexem::Op(Operand::from(ch).unwrap())
}

#[test]
fn test_parse() {
    let v = parse::<f64>("y8 - x + y/(8*6.38 - 5)-5x+y8".to_string());
    v.iter().fold(0, |s, l| {
        println!("{:?}", l);
        s + 1
    });
}

