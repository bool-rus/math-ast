use super::tree::*;
use super::lexem::*;
use std::fmt::Debug;
use super::num::Float;
use std::str::FromStr;
use parser::faces::Function;
use std::collections::HashMap;
use parser::faces::FnFunction;

#[derive(Debug)]
pub struct BuilderErr (
    String,
    Option<Builder>,
);

type BuildResult = Result<Builder, BuilderErr>;
type BB = Box<Builder>;

impl Into<BuildResult> for Builder {
    fn into(self) -> BuildResult {
        Ok(self)
    }
}

impl<O> Into<Result<O, BuilderErr>> for BuilderErr {
    fn into(self) -> Result<O, BuilderErr> {
        Err(self)
    }
}

#[derive(Debug)]
pub enum Builder {
    Empty,
    Simple(Lexem),
    Complex(Operand, BB, BB),
    Body(BB),
    Complete(BB),
    Fun(String, Vec<Builder>),
}


impl Builder {
    fn sin<T:Float>(args: Vec<T>) -> T {
        args[0].sin()
    }
    fn cos<T:Float>(args: Vec<T>) -> T {
        args[0].cos()
    }

    pub fn new() -> Builder {
        Builder::Empty
    }

    fn functions<T: 'static + Float + Sized>() -> HashMap<String, Box<Function<T>>> {
        let mut map = HashMap::new();
        map.insert("sin".to_string(), FnFunction::new("sin", &Self::sin));
        map.insert("cos".to_string(), FnFunction::new("cos", &Self::cos));
        map
    }

    fn simple_err<X, S: ToString>(s: S) -> Result<X, BuilderErr> {
        BuilderErr(s.to_string(), None).into()
    }
    fn make_err<X, S: ToString>(self, s: S) -> Result<X, BuilderErr> {
        BuilderErr(s.to_string(), Some(self)).into()
    }
    pub fn ast<T: 'static + Float + Sized>(self) -> Result<Ast<T>, BuilderErr> where T: Float {
        Ok(match self {
            Builder::Empty => return Self::simple_err("expression not complete"),
            Builder::Simple(Lexem::Letter(inner)) => match T::from_str_radix(&inner, 10) {
                Ok(num) => Ast::Constant(num),
                Err(_) => Ast::Variable(inner),
            },
            Builder::Simple(_) => unreachable!(),
            Builder::Complex(op, a, b) => Ast::Operation(op.into(), vec![a.ast()?, b.ast()?]),
            b @ Builder::Body(..) => b.make_err("expected ')'")?,
            Builder::Fun(..) => unreachable!(), //см обработку Complete
            Builder::Complete(inner) => inner.ast_inner()?,
        })
    }
    fn ast_inner<T: 'static + Float + Sized>(self) -> Result<Ast<T>,BuilderErr> {
        match self {
            Builder::Fun(name,v) => {
                let fun = match Self::functions().remove(&name) {
                    None => Self::simple_err(format!("Function {} not found", name))?,
                    Some(x) => x,
                };
                let mut builders = Vec::with_capacity(v.len());
                for b in v {
                    builders.push(b.ast()?);
                }
                Ok(Ast::Operation(fun, builders))
            },
            b => b.ast(),
        }
    }
    fn want_process(&self, lex: &Lexem) -> bool {
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
    pub fn process(self, lex: Lexem) -> BuildResult {
        #[cfg(test)] println!("lex: {:?}, b: {:?}", lex, self);
        match (self, lex) {
            (Builder::Empty, lex @ Lexem::Letter(_)) => Builder::Simple(lex),
            (Builder::Empty, Lexem::Open) => Builder::Body(Builder::Empty.into()),
            (a @ Builder::Complete(..), Lexem::Op(op)) |
            (a @ Builder::Simple(..), Lexem::Op(op)) => Builder::Complex(op, a.into(), Builder::Empty.into()),
            (Builder::Simple(Lexem::Letter(fun)), Lexem::Open) => Builder::Fun(fun, vec![Builder::Empty]),
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
    fn fun_delegate_inner(name: String, mut v: Vec<Builder>, lex: Lexem) -> BuildResult {
        let inner = v.pop().unwrap().process(lex)?;
        v.push(inner);
        Ok(Builder::Fun(name, v))
    }
}

impl FromStr for Builder {
    type Err = BuilderErr;

    fn from_str(s: &str) -> BuildResult {
        parse(s).into_iter().fold(Ok(Builder::new()), |b, lex| b?.process(lex))
    }
}


#[cfg(test)]
mod test {
    use parser::lexem::Lexem;
    use std::collections::HashMap;
    use parser::lexem::Operand;
    use parser::builder::Builder;
    use parser::builder::BuildResult;
    use super::super::num::Float;
    use std::fmt::Debug;
    use std::fmt::Display;
    use std::str::FromStr;

    #[test]
    fn build_plus() {
        let b = Builder::from_str("x + 10").unwrap();
        let tree = b.ast().unwrap();
        println!("tree: {:?}", tree);
        let mut params = HashMap::new();
        params.insert("x".to_string(), 5f64);

        let res = tree.calculate(&params).unwrap();
        assert_eq!(15f64, res);
    }

    #[test]
    fn build_ord() {
        let b = Builder::from_str("x-10*x+9").unwrap();
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
        let b = Builder::from_str("x - 10+9").unwrap();
        let tree = b.ast().unwrap();
        println!("tree: {:?}", tree);
        let mut params = HashMap::new();
        params.insert("x".to_string(), x);

        let res = tree.calculate(&params).unwrap();
        assert_eq!(x - 10.0 + 9.0, res);
    }

    #[test]
    fn build_brackets() {
        let x = 1f64;
        let b = Builder::from_str("x -3*(x-2)").unwrap();
        let tree = b.ast().unwrap();
        println!("tree: {:?}", tree);
        let mut params = HashMap::new();


        params.insert("x".to_string(), x);

        let res = tree.calculate(&params).unwrap();
        assert_eq!(x-3.0*(x-2.0), res);
    }

    #[test]
    fn build_power() {
        let x = 2f64;
        let b = Builder::from_str("x -3^(x-2)").unwrap();
        let tree = b.ast().unwrap();
        println!("tree: {:?}", tree);
        let mut params = HashMap::new();
        params.insert("x".to_string(), x);

        let res = tree.calculate(&params).unwrap();
        assert_eq!(x-3.0.powf(x-2.0), res);
    }

    #[test]
    fn build_sin() {
        //5*sin(x)

        let x = 0.3;
        let b = Builder::from_str("5*sin(x)").unwrap();
        let mut params = HashMap::new();
        params.insert("x".to_string(), x);

        let expected = Builder::Complex(Operand::High('*'),
                                        Builder::Simple(Lexem::Letter("5".to_string())).into(),
                                        Builder::Complete(
                                            Builder::Fun("sin".to_string(), vec![
                                                Builder::Simple(Lexem::Letter("x".to_string()))
                                            ]).into()
                                        ).into()
        );
        assert_eq!(format!("{:?}",b), format!("{:?}",expected))
    }

    #[test]
    fn multiple_fun() {
        let x = 0.3;
        let b = Builder::from_str("sin(x)^2 +cos(x)^2").unwrap();
        let mut params = HashMap::new();
        let ast = b.ast().unwrap();
        params.insert("x".to_string(), x);
        assert_eq!(1.0, ast.calculate(&params).unwrap());
    }
}