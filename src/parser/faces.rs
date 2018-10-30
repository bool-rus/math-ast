use std::fmt::{Debug,Formatter,Result};
use std::ops::Deref;
use super::num::Float;


pub trait Function<T: Sized> : Debug {
    fn name(&self) -> &str;
    fn args_count(&self) -> usize;
    fn call(&self, args: Vec<T>) -> T;
}

pub struct FnFunction<T: 'static + Sized> {
    name: String,
    args_count: usize,
    fun: &'static Fn(Vec<T>)->T,
}

impl<T: 'static + Sized> Function<T> for FnFunction<T> {
    fn name(&self) -> &str {
        &self.name
    }
    fn args_count(&self) -> usize {
        self.args_count.clone()
    }
    fn call(&self, args: Vec<T>) -> T {
        (self.fun)(args)
    }
}

impl<T> Debug for FnFunction<T> {
    fn fmt<'a>(&self, f: &mut Formatter<'a>) -> Result {
        write!(f,"{}", self.name)
    }
}

impl<T: Sized> FnFunction<T> {
    pub fn new<S: ToString>(name:S, func: &'static Fn(Vec<T>)->T) -> Box<Function<T>> {
        Box::new(FnFunction {
            name: name.to_string(),
            args_count: 2,
            fun: func
        })
    }
}