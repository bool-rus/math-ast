use std::fmt::{Debug,Formatter,Result};
use std::ops::Deref;

pub struct Fun<T>(String,Box<Fn(Vec<T>)->T>);

impl<T> Debug for Fun<T> {
    fn fmt<'a>(&self, f: &mut Formatter<'a>) -> Result {
        write!(f,"{}", self.0)
    }
}

impl<T> Deref for Fun<T> {
    type Target = Box<Fn(Vec<T>)->T>;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<T> Fun<T> {
    pub fn new<S: ToString,X: 'static + Fn(Vec<T>)->T>(name:S, x: X) -> Self {
        Fun(name.to_string(), Box::new(x))
    }
}