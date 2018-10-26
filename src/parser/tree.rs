extern crate std;

use std::collections::HashMap;
use std::ops::{Add, Neg, Mul, Div, Sub};
use parser::faces::Fun;

pub type BAst<T> = Box<Ast<T>>;

impl<T,E> Into<Result<BAst<T>,E>> for BAst<T> {
    fn into(self) -> Result<Box<Ast<T>>, E> {
        Ok(self)
    }
}


//*
#[derive(Debug)]
pub enum Ast<T> {
    Constant(T),
    Variable(String),
    Operation(Fun<T>, BAst<T>, BAst<T>),
}

impl<T> Ast<T> where T: Clone + Add<T, Output=T> + Mul<T,Output=T> + Sub<T, Output=T> + Div<T, Output = T>{
    pub fn calculate(&self, params: &HashMap<String, T>) -> Option<T> {
        Some(match self {
            Ast::Constant(num) => num.clone(),
            Ast::Variable(name) => params.get(name)?.clone(),
            Ast::Operation(fun,a,b) => fun(vec![a.calculate(params)?, b.calculate(params)?]),
        })
    }
}
//*/


#[cfg(test)]
mod test {
    use parser::tree::BAst;
    use parser::tree::Ast;
    use parser::faces::Fun;
    use std::collections::HashMap;
    use std::ops::{Add,Sub,Mul,Div};
    use parser::lexem::Operand;
    use super::super::num::Float;

    fn plus<T>(a: BAst<T>, b: BAst<T>) -> BAst<T> where T: Float {
        Box::new(Ast::Operation(Operand::make_fn::<T>('+'),a, b))
    }
    fn minus<T>(a: BAst<T>, b: BAst<T>) -> BAst<T> where T: Float {
        Box::new(Ast::Operation(Operand::make_fn::<T>('-'),a, b))
    }
    fn multiple<T>(a: BAst<T>, b: BAst<T>) -> BAst<T> where T: Float {
        Box::new(Ast::Operation(Operand::make_fn::<T>('*'),a, b))
    }
    fn divide<T>(a: BAst<T>, b: BAst<T>) -> BAst<T> where T: Float {
        Box::new(Ast::Operation(Operand::make_fn::<T>('/'),a, b))
    }
    fn constant<T>(t: T) -> BAst<T> {
        Box::new(Ast::Constant(t))
    }
    fn variable<T>(name: String) -> BAst<T> {
        Box::new(Ast::Variable(name))
    }
    #[test]
    fn test_tree() {
        let x = Box::new(constant(1f64));
        x.calculate(&HashMap::new());
        let tree = plus(
            constant(60f64),
            multiple(
                           minus(
                               constant(12f64),
                               divide(
                                   constant(10f64),
                                   variable("x".to_string())
                               )
                           ),
                           variable("y".to_string())
            ).into()
        ); // 60 + (12-10/x)*y
        println!("tree: {:?}", tree);
        let mut map = HashMap::new();
        map.insert("x".to_string(), 5f64);
        map.insert("y".to_string(), 2f64);
        assert_eq!(tree.calculate(&map).unwrap(), 80f64);

        map.insert("x".to_string(), 2f64);
        map.insert("y".to_string(), 5f64);
        assert_eq!(tree.calculate(&map).unwrap(), 95f64);
    }
}