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
            Ast::Operation(fun,a,b) => fun(a.calculate(params)?, b.calculate(params)?),
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

    fn plus<T>(a: BAst<T>, b: BAst<T>) -> BAst<T> where T: Add<T, Output=T> {
        Box::new(Ast::Operation(Fun::from(|x,y|x+y),a, b))
    }
    fn minus<T>(a: BAst<T>, b: BAst<T>) -> BAst<T> where T: Sub<T, Output=T> {
        Box::new(Ast::Operation(Fun::from(|x,y|x-y),a, b))
    }
    fn multiple<T>(a: BAst<T>, b: BAst<T>) -> BAst<T> where T: Mul<T, Output=T> {
        Box::new(Ast::Operation(Fun::from(|x,y|x*y),a, b))
    }
    fn divide<T>(a: BAst<T>, b: BAst<T>) -> BAst<T> where T: Div<T, Output=T> {
        Box::new(Ast::Operation(Fun::from(|x,y|x/y),a, b))
    }
    fn constant<T>(t: T) -> BAst<T> {
        Box::new(Ast::Constant(t))
    }
    fn variable<T>(name: String) -> BAst<T> {
        Box::new(Ast::Variable(name))
    }
    #[test]
    fn test_tree() {
        let x = Box::new(constant(1));
        x.calculate(&HashMap::new());
        let tree = plus(
            constant(60),
            multiple(
                           minus(
                               constant(12),
                               divide(
                                   constant(10),
                                   variable("x".to_string())
                               )
                           ),
                           variable("y".to_string())
            ).into()
        ); // 60 + (12-10/x)*y
        println!("tree: {:?}", tree);
        let mut map = HashMap::new();
        map.insert("x".to_string(), 5);
        map.insert("y".to_string(), 2);
        assert_eq!(tree.calculate(&map).unwrap(), 80);

        map.insert("x".to_string(), 2);
        map.insert("y".to_string(), 5);
        assert_eq!(tree.calculate(&map).unwrap(), 95);
    }
}