use std::collections::HashMap;

use std::ops::{Add, Neg, Mul, Div, Sub};

pub type BAst<T> = Box<Ast<T>>;

impl<T,E> Into<Result<BAst<T>,E>> for BAst<T> {
    fn into(self) -> Result<Box<Ast<T>>, E> {
        Ok(self)
    }
}


//*
#[derive(Debug)]
pub enum Ast<T> {
    Plus(BAst<T>, BAst<T>),
    Minus(BAst<T>, BAst<T>),
    Multiple(BAst<T>, BAst<T>),
    Divide(BAst<T>, BAst<T>),
    Constant(T),
    Variable(String),
}

impl<T> Ast<T> where T: Clone + Add<T, Output=T> + Mul<T,Output=T> + Sub<T, Output=T> + Div<T, Output = T>{
    pub fn calculate(&self, params: &HashMap<String, T>) -> Option<T> {
        Some(match self {
            Ast::Constant(num) => num.clone(),
            Ast::Variable(name) => params.get(name)?.clone(),
            Ast::Plus(a, b) => a.calculate(params)? + b.calculate(params)?,
            Ast::Minus(a, b) => a.calculate(params)? - b.calculate(params)?,
            Ast::Multiple(a, b) => a.calculate(params)? * b.calculate(params)?,
            Ast::Divide(a, b) => a.calculate(params)? / b.calculate(params)?,
        })
    }
    pub fn plus(a: BAst<T>, b: BAst<T>) -> BAst<T> {
        Box::new(Ast::Plus(a, b))
    }
    pub fn minus(a: BAst<T>, b: BAst<T>) -> BAst<T> {
        Box::new(Ast::Minus(a, b))
    }
    pub fn multiple(a: BAst<T>, b: BAst<T>) -> BAst<T> {
        Box::new(Ast::Multiple(a, b))
    }
    pub fn divide(a: BAst<T>, b: BAst<T>) -> BAst<T> {
        Box::new(Ast::Divide(a, b))
    }
    pub fn constant(t: T) -> BAst<T> {
        Box::new(Ast::Constant(t))
    }
    pub fn variable(name: String) -> BAst<T> {
        Box::new(Ast::Variable(name))
    }
}
//*/



#[test]
fn test_tree() {
    let x = Box::new(Ast::Constant(1));
    x.calculate(&HashMap::new());
    let tree = Ast::plus(
        Ast::constant(60),
        Ast::multiple(
            Ast::minus(
                Ast::constant(12),
                Ast::divide(
                    Ast::constant(10),
                    Ast::variable("x".to_string())
                    )
                ),
            Ast::variable("y".to_string())
            )
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