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
pub struct BuilderErr<T> (
    String,
    Option<Builder<T>>,
);

type BuildResult<T> = Result<Builder<T>, BuilderErr<T>>;
type BB<T> = Box<Builder<T>>;

impl<T> Into<BuildResult<T>> for Builder<T> {
    fn into(self) -> BuildResult<T> {
        Ok(self)
    }
}
impl<T,O> Into<Result<O, BuilderErr<T>>> for BuilderErr<T> {
    fn into(self) -> Result<O, BuilderErr<T>> {
        Err(self)
    }
}

#[derive(Debug)]
pub enum Builder<T> {
    Empty,
    Simple(BAst<T>),
    Op(BB<T>, Operand),
    Complex(BB<T>, Operand, BB<T>),
    Body(BB<T>),
}

impl<T> Builder<T> where T: Debug + Clone + Add<T, Output=T> + Mul<T, Output=T> + Sub<T, Output=T> + Div<T, Output=T> {
    pub fn new() -> Builder<T> {
        Builder::Empty
    }
    fn simple_err<X>(s: &'static str) -> Result<X,BuilderErr<T>>{
        BuilderErr(s.into(),None).into()
    }
    fn make_err<X>(self, s: &'static str) -> Result<X, BuilderErr<T>> {
        BuilderErr(s.into(), Some(self)).into()
    }
    pub fn ast(self) -> Result<BAst<T>,BuilderErr<T>> {
        match self {
            Builder::Empty => Self::simple_err("expression not complete"),
            Builder::Simple(bast) => bast.into(),
            b @ Builder::Op(..) => b.make_err("expression not complete: operation not closed"),
            Builder::Complex( a, op, b) => {
                let (a, b) = (a.ast()?, b.ast()?);
                match op {
                    Operand::Plus => Ast::plus(a, b).into(),
                    Operand::Minus => Ast::minus(a, b).into(),
                    Operand::Multiple => Ast::multiple(a, b).into(),
                    Operand::Divide => Ast::divide(a, b).into(),
                    _ => BuilderErr(format!("{:?} in complex",op), None).into(),
                }
            }
            b @ Builder::Body(..) => b.make_err("expected ')'").into()
        }
    }
    pub fn process(self, lex: Lexem<T>) -> BuildResult<T> {
        match self {
            Builder::Empty => Self::for_empty(lex),
            a @ Builder::Simple(..) => Self::for_simple(a.into(), lex),
            Builder::Op(a, op) => Self::for_op(a, op, lex),
            Builder::Complex(a, op, b) => Self::for_complex(a, op, b, lex),
            Builder::Body(inner) => Self::for_body(inner,lex),
        }.into()
    }
    fn for_empty(lex: Lexem<T>) -> BuildResult<T> {
        match lex {
            Lexem::Number(num) => Builder::Simple(Ast::constant(num)),
            Lexem::Letter(name) => Builder::Simple(Ast::variable(name)),
            Lexem::Op(Operand::Open) => Builder::Body(Builder::Empty.into()),
            _ => return BuilderErr(format!("unexpected lexem: {:?}", lex),None).into()
        }.into()
    }
    fn for_simple(a: BB<T>, lex: Lexem<T>) -> BuildResult<T> { //TOD: потом доделать для функции
        match lex {
            Lexem::Op(op) =>
                Builder::Op(
                    a,
                    op,
                ),
            l => return BuilderErr(format!("expected lexem after var/const, found {:?}", l),Some(*a)).into()
        }.into()
    }
    fn for_op(a: BB<T>, op: Operand, lex: Lexem<T>) -> BuildResult<T> {
        Builder::Complex(a, op, Self::for_empty(lex)?.into()).into()
    }
    fn for_complex(a: BB<T>, op: Operand, b: BB<T>, lex: Lexem<T>) -> BuildResult<T> {
        match (*b, lex) {
            (Builder::Empty, _) => unreachable!(),
            (b @ Builder::Body(..), lex) |
            (b @ Builder::Op(..), lex) => Ok(
                Builder::Complex(a, op, b.process(lex)?.into())
            ),
            (Builder::Simple(..), Lexem::Op(Operand::Open)) => unreachable!(), //доделать для функции
            (b @ Builder::Simple(..), Lexem::Op(new_op)) => Ok({
                let b: BB<T> = b.into();
                if new_op > op {
                    Builder::Complex(a, op, Builder::Op(b, new_op).into())
                } else {
                    Builder::Op(Builder::Complex( a, op, b).into(), new_op)
                }
            }),
            (b @ Builder::Complex(..), Lexem::Op(new_op)) => Ok({
                if new_op > op {
                    Builder::Complex(a, op, b.process(Lexem::Op(new_op))?.into())
                } else {
                    Builder::Op(Builder::Complex( a, op, b.into()).into(), new_op)
                }
            }),
            _ => Self::simple_err("called method for_complex, but object is simple")
        }
    }
    fn for_body(inner: BB<T>, lex: Lexem<T>) -> BuildResult<T> {
        match &lex {
            &Lexem::Op(Operand::Close) => Builder::Simple(inner.ast()?),
            _ => Builder::Body(inner.process(lex)?.into()),
        }.into()
    }
}

#[test]
fn build_plus() {
    let lexes = vec![
        Lexem::Letter("x".to_string()),
        Lexem::Op(Operand::Plus),
        Lexem::Number(10),
    ];
    let b = lexes.into_iter().fold(Builder::new(), |b, lex| b.process(lex).unwrap());
    let tree = b.ast().unwrap();
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
    let b = lexes.into_iter().fold(Builder::new(), |b, lex| b.process(lex).unwrap());
    let tree = b.ast().unwrap();
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
    let b = lexes.into_iter().fold(Builder::new(), |b, lex| b.process(lex).unwrap());
    let tree = b.ast().unwrap();
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
        Lexem::Op(Operand::Close),
        Lexem::Op(Operand::Multiple),
        Lexem::Number(3),
    ]; //x-(x-2)*3
    let b = lexes.into_iter().fold(Builder::new(), |b, lex| b.process(lex).unwrap());
    let tree = b.ast().unwrap();
    println!("tree: {:?}", tree);
    let mut params = HashMap::new();
    params.insert("x".to_string(), 1);

    let res = tree.calculate(&params).unwrap();
    assert_eq!(4, res);
}
