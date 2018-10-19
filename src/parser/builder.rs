use super::tree::*;
use super::lexem::*;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
use std::fmt::Debug;

#[cfg(test)]
use std::collections::HashMap;

#[derive(Debug)]
struct BuilderErr<T> {
    msg: String,
    result: Option<Box<Builder<T>>>,
}

type BuildResult<T> = Box<Builder<T>>;

impl<T> Into<Result<Builder<T>,BuilderErr<T>>> for Builder<T> {
    fn into(self) -> Result<Builder<T>, BuilderErr<T>> {
        Ok(self)
    }
}


#[derive(Debug)]
pub enum Builder<T> {
    Empty(bool),
    Simple(bool, BAst<T>),
    Op(bool, BuildResult<T>, Operand),
    Complex(bool, BuildResult<T>, Operand, BuildResult<T>),
}

impl<T> Builder<T> where T: Debug + Clone + Add<T, Output=T> + Mul<T, Output=T> + Sub<T, Output=T> + Div<T, Output=T> {
    pub fn new() -> Builder<T> {
        Builder::Empty(false)
    }
    pub fn ast(self) -> BAst<T> {
        match self {
            Builder::Empty(..) => unreachable!("expression not complete"),
            Builder::Simple(nested, ..) if nested => unreachable!("expected ')'"),
            Builder::Complex(nested, ..) if nested => unreachable!("expected ')'"),
            Builder::Simple(_, bast) => bast,
            Builder::Op(..) => unreachable!("expression not complete"),
            Builder::Complex(_, a, op, b) => {
                let (a, b) = (a.ast(), b.ast());
                match op {
                    Operand::Plus => Ast::plus(a, b),
                    Operand::Minus => Ast::minus(a, b),
                    Operand::Multiple => Ast::multiple(a, b),
                    Operand::Divide => Ast::divide(a, b),
                    _ => unreachable!("{:?} in complex")
                }
            }
        }
    }
    pub fn process(self, lex: Lexem<T>) -> Builder<T> {
        match self {
            Builder::Empty(nested) => Self::for_empty(nested, lex),
            a @ Builder::Simple(_, _) => Self::for_simple(a, lex),
            Builder::Op(nested, a, op) => Self::for_op(nested, a, op, lex),
            Builder::Complex(nested, a, op, b) => Self::for_complex(nested, a, op, b, lex),
        }
    }
    fn for_empty(nested: bool, lex: Lexem<T>) -> Builder<T> {
        match lex {
            Lexem::Number(num) => Builder::Simple(nested, Ast::constant(num)),
            Lexem::Letter(name) => Builder::Simple(nested, Ast::variable(name)),
            Lexem::Op(Operand::Open) => Builder::Empty(true),
            _ => unreachable!("unexpected lexem: {:?}", lex),
        }
    }
    fn for_simple(a: Builder<T>, lex: Lexem<T>) -> Builder<T> {
        match (a, lex) {
            (Builder::Simple(nested, ast), Lexem::Op(op)) =>
                Builder::Op(
                    nested,
                    Builder::Simple(false, ast).into(),
                    op,
                ),
            (_, l) => unreachable!("expected lexem after var/const, found {:?}", l)
        }
    }
    fn for_op(nested: bool, a: BuildResult<T>, op: Operand, lex: Lexem<T>) -> Builder<T> {
        Builder::Complex(nested, a, op, Self::for_empty(nested, lex).into())
    }
    fn for_complex(nested: bool, a: BuildResult<T>, op: Operand, b: BuildResult<T>, lex: Lexem<T>) -> Builder<T> {
        match (*b, lex) {
            (b @ Builder::Empty(..), lex) |
            (b @ Builder::Op(..), lex) => Builder::Complex(nested, a, op, b.process(lex).into()),
            (b, Lexem::Op(Operand::Close)) => match nested {
                true => Builder::Complex(false, a, op, b.into()),
                false => unreachable!("unexpected ')'"),
            },
            (b @ Builder::Simple(..), Lexem::Op(new_op)) => {
                let b = b.into();
                if new_op > op {
                    Builder::Complex(nested, a, op, Builder::Op(false, b, new_op).into())
                } else {
                    Builder::Op(nested, Builder::Complex(false, a, op, b).into(), new_op)
                }
            }
            (b @ Builder::Complex(..), Lexem::Op(new_op)) => {
                if new_op > op {
                    Builder::Complex(nested, a, op, b.process(Lexem::Op(new_op)).into())
                } else {
                    Builder::Op(nested, Builder::Complex(false, a, op, b.into()).into(), new_op)
                }
            }
            _ => unreachable!("called method for_complex, but object is simple")
        }
    }
}

#[test]
fn build_plus() {
    let lexes = vec![
        Lexem::Letter("x".to_string()),
        Lexem::Op(Operand::Plus),
        Lexem::Number(10),
    ];
    let b = lexes.into_iter().fold(Builder::new(), |b, lex| b.process(lex));
    let tree = b.ast();
    println!("tree: {:?}", tree);
    let mut params = HashMap::new();
    params.insert("x".to_string(), 5);

    let res = tree.calculate(&params).unwrap();
    assert_eq!(15, res);
}

#[test]
fn build_ord() {
    let lexes = vec![
        Lexem::Letter("x".to_string()),
        Lexem::Op(Operand::Minus),
        Lexem::Number(10),
        Lexem::Op(Operand::Multiple),
        Lexem::Letter("x".to_string()),
        Lexem::Op(Operand::Plus),
        Lexem::Number(9)
    ]; //x-10*x+9
    let b = lexes.into_iter().fold(Builder::new(), |b, lex| b.process(lex));
    let tree = b.ast();
    println!("tree: {:?}", tree);
    let mut params = HashMap::new();
    params.insert("x".to_string(), 1);

    let res = tree.calculate(&params).unwrap();
    assert_eq!(0, res);
}


#[test]
fn build_mipl() {
    let lexes = vec![
        Lexem::Letter("x".to_string()),
        Lexem::Op(Operand::Minus),
        Lexem::Number(10),
        Lexem::Op(Operand::Minus),
        Lexem::Letter("x".to_string()),
        Lexem::Op(Operand::Plus),
        Lexem::Number(9)
    ]; //x-10-x+9
    let b = lexes.into_iter().fold(Builder::new(), |b, lex| b.process(lex));
    let tree = b.ast();
    println!("tree: {:?}", tree);
    let mut params = HashMap::new();
    params.insert("x".to_string(), 1);

    let res = tree.calculate(&params).unwrap();
    assert_eq!(-1, res);
}

#[test]
fn build_brackets() {
    let lexes = vec![
        Lexem::Letter("x".to_string()),
        Lexem::Op(Operand::Minus),
        Lexem::Op(Operand::Open),
        Lexem::Letter("x".to_string()),
        Lexem::Op(Operand::Minus),
        Lexem::Number(2),
        Lexem::Op(Operand::Close)
    ]; //x-(x-2)
    let b = lexes.into_iter().fold(Builder::new(), |b, lex| b.process(lex));
    let tree = b.ast();
    println!("tree: {:?}", tree);
    let mut params = HashMap::new();
    params.insert("x".to_string(), 1);

    let res = tree.calculate(&params).unwrap();
    assert_eq!(2, res);
}

#[test]
fn test_from() {
    //let b = Builder::Empty(false);
    //let b = make_bb(b);
}