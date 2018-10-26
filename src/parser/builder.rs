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

impl<T, O> Into<Result<O, BuilderErr<T>>> for BuilderErr<T> {
    fn into(self) -> Result<O, BuilderErr<T>> {
        Err(self)
    }
}

#[derive(Debug)]
pub enum Builder<T> {
    Empty,
    Simple(Lexem<T>),
    Complex(Operand, BB<T>, BB<T>),
    Body(BB<T>),
    Complete(BB<T>),
    Fun(Lexem<T>, Vec<Builder<T>>),
}


impl<T> Builder<T> where T: Float + Debug {
    pub fn new() -> Builder<T> {
        Builder::Empty
    }
    fn simple_err<X, S: ToString>(s: S) -> Result<X, BuilderErr<T>> {
        BuilderErr(s.to_string(), None).into()
    }
    fn make_err<X, S: ToString>(self, s: S) -> Result<X, BuilderErr<T>> {
        BuilderErr(s.to_string(), Some(self)).into()
    }
    pub fn ast(self) -> Result<BAst<T>, BuilderErr<T>> {
        Ok(match self {
            Builder::Empty => return Self::simple_err("expression not complete"),
            Builder::Simple(Lexem::Letter(name)) => Ast::Variable(name),
            Builder::Simple(Lexem::Number(num)) => Ast::Constant(num),
            Builder::Simple(_) => unreachable!(),
            Builder::Complex(op, a, b) => Ast::Operation(op.into(), a.ast()?, b.ast()?),
            b @ Builder::Body(..) => return b.make_err("expected ')'"),
            b @ Builder::Fun(..) => unimplemented!(),
            Builder::Complete(b) => return b.ast(),
        }.into())
    }
    fn want_process(&self, lex: &Lexem<T>) -> bool {
        match (self, lex) {
            (Builder::Body(..), _) => true,
            (Builder::Fun(..), _) => true,
            (Builder::Empty, _) => true,
            (Builder::Simple(..), Lexem::Open) => true,
            (Builder::Complex(_, _, b), _) if b.want_process(lex) => true,
            (Builder::Complex(op, ..), Lexem::Op(new_op)) if new_op.more(op) => true,
            _ => false,
        }
    }
    pub fn process(self, lex: Lexem<T>) -> BuildResult<T> {
        #[cfg(test)] println!("lex: {:?}, b: {:?}", lex, self);
        match (self, lex) {
            (Builder::Empty, lex @ Lexem::Number(_)) |
            (Builder::Empty, lex @ Lexem::Letter(_)) => Builder::Simple(lex),
            (Builder::Empty, Lexem::Open) => Builder::Body(Builder::Empty.into()),
            (a @ Builder::Complete(..), Lexem::Op(op)) |
            (a @ Builder::Simple(..), Lexem::Op(op)) => Builder::Complex(op, a.into(), Builder::Empty.into()),
            (Builder::Simple(fun), Lexem::Open) => Builder::Fun(fun, vec![Builder::Empty]),
            (Builder::Complex(op, a, b), Lexem::Op(new_op)) => if b.want_process(&Lexem::Op(new_op)) || new_op.more(&op) {
                Builder::Complex(op, a, b.process(Lexem::Op(new_op))?.into())
            } else {
                Builder::Complex(new_op, Builder::Complex(op, a, b).into(), Builder::Empty.into())
            },
            (Builder::Complex(op, a, b), lex) => Builder::Complex(op,a,b.process(lex)?.into()),
            (Builder::Body(inner), lex @ Lexem::Close) => if inner.want_process(&lex) {
                Builder::Body(inner.process(lex)?.into())
            } else {
                Builder::Complete(inner)
            },
            (Builder::Body(inner), lex) => Builder::Body(inner.process(lex)?.into()),
            (Builder::Fun(name, v), lex @ Lexem::Close) => {
                if v.last().unwrap().want_process(&lex) {
                    Self::fun_delegate_inner(name, v, lex)?
                } else {
                    Builder::Complete(Builder::Fun(name, v).into())
                }
            },
            (Builder::Fun(name, mut v), lex @ Lexem::Comma) => {
                if v.last().unwrap().want_process(&lex) {
                    Self::fun_delegate_inner(name, v, lex)?
                } else {
                    v.push(Builder::Empty);
                    Builder::Fun(name, v)
                }
            },
            (Builder::Fun(name, v), lex) => Self::fun_delegate_inner(name, v, lex)?,
            (b, lex) => return b.make_err(format!("Unexpected lexem {:?}", lex)),
        }.into()
    }
    fn fun_delegate_inner(name: Lexem<T>, mut v: Vec<Builder<T>>, lex: Lexem<T>) -> BuildResult<T> {
        let inner = v.pop().unwrap().process(lex)?;
        v.push(inner);
        Ok(Builder::Fun(name, v))
    }
}


#[cfg(test)]
mod test {
    use parser::lexem::Lexem;
    use parser::lexem::make_operand;
    use std::collections::HashMap;
    use parser::lexem::Operand;
    use parser::builder::Builder;
    use parser::builder::BuildResult;
    use super::super::num::Float;
    use std::fmt::Debug;

    fn parse_lexemes<T: Float + Debug>(v: Vec<Lexem<T>>) -> BuildResult<T> {
        v.into_iter().fold(Ok(Builder::Empty), |b, lex| Ok(b?.process(lex)?))
    }

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
        assert_eq!(x - f10 - x + f9, res);
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

    #[test]
    fn build_sin() {
        //sin(x)
        let lexes = vec![
            Lexem::Letter("sin".to_string()),
            Lexem::Open,
            Lexem::Letter("x".to_string()),
            Lexem::Close,
        ];
        let x = 0.3;
        let mut params = HashMap::new();
        params.insert("x".to_string(), x);

        let b = parse_lexemes(lexes).unwrap();
        let tree = b.ast().unwrap();
        assert_eq!(x.sin(), tree.calculate(&params).unwrap());
    }
}