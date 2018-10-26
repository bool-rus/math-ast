use super::tree::*;
use super::lexem::*;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
use std::fmt::Debug;
use super::num::Float;

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
    Pending(BB<T>, Operand),
    Complete(BB<T>, Operand, BB<T>),
    Body(BB<T>),
}


impl<T> Builder<T> where T: Float + Debug {
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
            Builder::Empty => return Self::simple_err("expression not complete"),
            Builder::Simple(bast) => bast,
            b @ Builder::Pending(..) => return b.make_err("expression not complete: operation not closed"),
            Builder::Complete(a, op, b) => Ast::Operation(op.into(), a.ast()?, b.ast()?).into(),
            b @ Builder::Body(..) => return b.make_err("expected ')'")
        }.into()
    }
    pub fn process(self, lex: Lexem<T>) -> BuildResult<T> {
        #[cfg(test)] println!("lex: {:?}, b: {:?}",lex, self);
        match self {
            Builder::Empty => Self::for_empty(lex),
            a @ Builder::Simple(..) => Self::for_simple(a.into(), lex),
            Builder::Pending(a, op) => Self::for_op(a, op, lex),
            Builder::Complete(a, op, b) => Self::for_complex(a, op, b, lex),
            Builder::Body(inner) => Self::for_body(inner,lex),
        }.into()
    }
    fn has_body(&self) -> bool {
        match &self {
            &Builder::Body(..) => true,
            &Builder::Complete(_, _, b) => b.has_body(),
            _ => false,
        }
    }
    fn for_empty(lex: Lexem<T>) -> BuildResult<T> {
        match lex {
            Lexem::Number(num) => Builder::Simple(Ast::Constant(num).into()),
            Lexem::Letter(name) => Builder::Simple(Ast::Variable(name).into()),
            Lexem::Open => Builder::Body(Builder::Empty.into()),
            _ => return BuilderErr(format!("unexpected lexem: {:?}", lex),None).into()
        }.into()
    }
    fn for_simple(a: BB<T>, lex: Lexem<T>) -> BuildResult<T> { //TOD: потом доделать для функции
        match lex {
            Lexem::Op(op) =>
                Builder::Pending(
                    a,
                    op,
                ),
            l => return BuilderErr(format!("expected lexem after var/const, found {:?}", l),Some(*a)).into()
        }.into()
    }
    fn for_op(a: BB<T>, op: Operand, lex: Lexem<T>) -> BuildResult<T> {
        Builder::Complete(a, op, Self::for_empty(lex)?.into()).into()
    }
    fn for_complex(a: BB<T>, op: Operand, b: BB<T>, lex: Lexem<T>) -> BuildResult<T> {
        match (*b, lex) {
            (Builder::Empty, _) => unreachable!(),
            (b @ Builder::Body(..), lex) |
            (b @ Builder::Pending(..), lex) => Ok(
                Builder::Complete(a, op, b.process(lex)?.into())
            ),
            (Builder::Simple(..), Lexem::Open) => unreachable!(), //доделать для функции
            (b @ Builder::Simple(..), Lexem::Op(new_op)) => Ok({
                let b: BB<T> = b.into();
                if new_op.more(&op) {
                    Builder::Complete(a, op, Builder::Pending(b, new_op).into())
                } else {
                    Builder::Pending(Builder::Complete(a, op, b).into(), new_op)
                }
            }),
            (b @ Builder::Complete(..), Lexem::Op(new_op)) => Ok({
                if new_op > op || b.has_body() {
                    Builder::Complete(a, op, b.process(Lexem::Op(new_op))?.into())
                } else {
                    Builder::Pending(Builder::Complete(a, op, b.into()).into(), new_op)
                }
            }),
            (b @ Builder::Complete(..), lex) => if b.has_body() {
                Builder::Complete(a, op, b.process(lex)?.into()).into()
            } else {
                Self::simple_err("unexpected lexem")
            }
            _ => unreachable!()
        }
    }
    fn for_body(inner: BB<T>, lex: Lexem<T>) -> BuildResult<T> {
        match &lex {
            &Lexem::Close if !inner.has_body() => Builder::Simple(inner.ast()?),
            _ => Builder::Body(inner.process(lex)?.into()),
        }.into()
    }
}



#[cfg(test)]
mod test {
    use parser::lexem::Lexem;
    use parser::lexem::make_operand;
    use std::collections::HashMap;
    use parser::lexem::Operand;
    use parser::builder::Builder;


    #[test]
    fn build_plus() {
        let lexes = vec![
            Lexem::Letter("x".to_string()),
            make_operand('+'),
            Lexem::Number(10f64),
        ];
        let b = lexes.into_iter().fold(Builder::new(), |b, lex| b.process(lex).unwrap());
        let tree = b.ast().unwrap();
        println!("tree: {:?}", tree);
        let mut params = HashMap::new();
        params.insert("x".to_string(), 5f64);

        let res = tree.calculate(&params).unwrap();
        assert_eq!(15f64, res);
    }

    #[test]
    fn build_ord() {
        let lexes = vec![
            Lexem::Letter("x".to_string()),
            make_operand('-'),
            Lexem::Number(10f64),
            make_operand('*'),
            Lexem::Letter("x".to_string()),
            make_operand('+'),
            Lexem::Number(9f64)
        ]; //x-10*x+9
        let b = lexes.into_iter().fold(Builder::new(), |b, lex| b.process(lex).unwrap());
        let tree = b.ast().unwrap();
        println!("tree: {:?}", tree);
        let mut params = HashMap::new();
        params.insert("x".to_string(), 1f64);

        let res = tree.calculate(&params).unwrap();
        assert_eq!(0f64, res);
    }
    #[test]
    fn build_mipl() {
        let x = 1f64;
        let f10 = 10f64;
        let f9 = 9f64;
        let lexes = vec![
            Lexem::Letter("x".to_string()),
            make_operand('-'),
            Lexem::Number(f10),
            make_operand('-'),
            Lexem::Letter("x".to_string()),
            make_operand('+'),
            Lexem::Number(f9)
        ]; //x-10-x+9
        let b = lexes.into_iter().fold(Builder::new(), |b, lex| b.process(lex).unwrap());
        let tree = b.ast().unwrap();
        println!("tree: {:?}", tree);
        let mut params = HashMap::new();
        params.insert("x".to_string(), x);

        let res = tree.calculate(&params).unwrap();
        assert_eq!(x-f10-x+f9, res);
    }

    #[test]
    fn build_brackets() {
        let lexes = vec![
            Lexem::Letter("x".to_string()),
            make_operand('-'),
            Lexem::Number(3f64),
            make_operand('*'),
            Lexem::Open,
            Lexem::Letter("x".to_string()),
            make_operand('-'),
            Lexem::Number(2f64),
            Lexem::Close,
        ]; //x-3*(x-2)
        let b = lexes.into_iter().fold(Builder::new(), |b, lex| b.process(lex).unwrap());
        let tree = b.ast().unwrap();
        println!("tree: {:?}", tree);
        let mut params = HashMap::new();
        params.insert("x".to_string(), 1f64);

        let res = tree.calculate(&params).unwrap();
        assert_eq!(4f64, res);
    }

    #[test]
    fn build_power() {
        let lexes = vec![
            Lexem::Letter("x".to_string()),
            make_operand('-'),
            Lexem::Number(3f64),
            make_operand('^'),
            Lexem::Open,
            Lexem::Letter("x".to_string()),
            make_operand('-'),
            Lexem::Number(2f64),
            Lexem::Close,
        ]; //x-3^(x-2)
        let b = lexes.into_iter().fold(Builder::new(), |b, lex| b.process(lex).unwrap());
        let tree = b.ast().unwrap();
        println!("tree: {:?}", tree);
        let mut params = HashMap::new();
        params.insert("x".to_string(), 2f64);

        let res = tree.calculate(&params).unwrap();
        assert_eq!(1_f64, res);
    }
}