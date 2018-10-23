
use std::str::FromStr;
use std::fmt::Debug;
use std::cmp::Ordering;
use std::ops::{Add,Sub,Mul,Div};
use parser::faces::Fun;

#[derive(Debug)]
pub enum Lexem<T> {
    Number(T),
    Letter(String),
    Op(Operand)
}

#[derive(Debug,Copy,Clone,PartialOrd,Eq,PartialEq)]
pub enum Operand {
/*
    Plus = 1,
    Minus = 2,
    Multiple = 4,
    Divide = 3,
*/
    Low(char),
    High(char),

    Open,
    Close,

}
/*
fn fun<T,F>(f: F, t: T) -> T where F: Into<Fn(T,T)->T>  {
    let f = f.into();
    f(t,t)
}
*/


impl Operand {
    pub fn more(&self, rhs: &Operand) -> bool{
        match (self, rhs) {
            (Operand::High(..),Operand::Low(..)) => true,
            _ => false,
        }
    }
    fn from(ch: char) -> Option<Operand> {
        match ch {
            '+' => Some(Operand::Low('+')),
            '-' => Some(Operand::Low('-')),
            '*' => Some(Operand::High('*')),
            '/' => Some(Operand::High('/')),
            '(' => Some(Operand::Open),
            ')' => Some(Operand::Close),
            _ => None,
        }
    }
    pub fn to_fn<T>(self) -> Fun<T>
        where //X: From<Fn(T,T)->T>,
              T: Debug + Clone + Add<T, Output=T> + Mul<T, Output=T> + Sub<T, Output=T> + Div<T, Output=T>
    {
        match self.ch() {
            '+' => Fun::from(|a, b| a+b),
            '-' => Fun::from(|a, b| a-b),
            '*' => Fun::from(|a, b| a*b),
            '/' => Fun::from(|a, b| a/b),
            _ => unreachable!(),
        }
    }
    fn ch(self) -> char{
        match self {
            Operand::Low(c) => c,
            Operand::High(c) => c,
            _ => unreachable!()
        }
    }
}
#[derive(Debug)]
enum State {
    None,
    Letter,
    Operand(Operand)
}

impl From<char> for State {
    fn from(ch: char) -> Self {
        let op = Operand::from(ch);
        if let Some(op) = op {
            State::Operand(op)
        } else {
            match ch {
                '0'|'.'|'1'...'9' | 'a'...'z' |'A'...'Z' => State::Letter,
                _ => State::None
            }
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


impl<T> Parser<T> where T: FromStr + Debug, T::Err: Debug {
    pub fn new() -> Parser<T> {
        Parser {
            state: State::None,
            lexemes: Vec::new(),
            buf: String::new(),
        }
    }
    pub fn process(mut self, ch: char) -> Self {
        let new_state = State::from(ch);
        match(self.state, new_state) {
            (prev, State::None) => self.state = prev,
            (_, s @ State::Letter) => {
                self.state = s;
                self.buf.push(ch);
            }
            (State::Letter, State::Operand(op)) => {
                self.state = State::Operand(op);
                let letter = self.buf;
                self.buf = String::new();
                self.end_letter(letter);
                self.lexemes.push(Lexem::Op(op));
            },
            (_, State::Operand(op)) => {
                self.state = State::Operand(op);
                self.lexemes.push(Lexem::Op(op));
            },

        };
        self
    }
    fn end_letter(&mut self, letter: String) {
        if letter.is_empty() {
            return
        }
        match letter.parse::<T>() {
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

pub fn parse<T>(input: String) -> Vec<Lexem<T>> where T: FromStr + Debug, T::Err: Debug {
    let p  = input.chars().fold(Parser::new(), |p, ch|p.process(ch)).end();
    p.lexemes
}


#[cfg(test)]
pub fn make_operand<T>(ch: char) -> Lexem<T> {
    Lexem::Op(Operand::from(ch).unwrap())
}

#[test]
fn test_parse() {
    let v = parse::<f64>("y8 - x + y/(8*6.38 - 5)-5x+y8".to_string());
    v.iter().fold(0,|s,l| {
        println!("{:?}",l);
        s+1
    });
}

