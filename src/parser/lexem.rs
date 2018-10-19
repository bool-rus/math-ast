
use std::str::FromStr;
use std::fmt::Debug;
use std::cmp::Ordering;

#[derive(Debug)]
pub enum Lexem<T> {
    Number(T),
    Letter(String),
    Op(Operand)
}

#[derive(Debug,Copy,Clone,PartialOrd,Eq,PartialEq)]
pub enum Operand {
    Plus = 1,
    Minus = 2,
    Multiple = 4,
    Divide = 3,
    Open = 5,
    Close = 6,
}


impl Operand {
    fn from_operand(ch: char) -> Option<Operand> {
        match ch {
            '+' => Some(Operand::Plus),
            '-' => Some(Operand::Minus),
            '*' => Some(Operand::Multiple),
            '/' => Some(Operand::Divide),
            '(' => Some(Operand::Open),
            ')' => Some(Operand::Close),
            _ => None,
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
        let op = Operand::from_operand(ch);
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

#[test]
fn test_parse() {
    let v = parse::<f64>("y8 - x + y/(8*6.38 - 5)-5x+y8".to_string());
    v.iter().fold(0,|s,l| {
        println!("{:?}",l);
        s+1
    });
}

