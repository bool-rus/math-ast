use std::fmt::{Debug,Formatter,Result};
use std::ops::Deref;

pub struct Fun<T>(Box<(Fn(T,T)->T)>);

impl<T> Debug for Fun<T> {
    fn fmt<'a>(&self, f: &mut Formatter<'a>) -> Result {
        write!(f,"fun")
    }
}

impl<T> Deref for Fun<T> {
    type Target = Box<(Fn(T,T)->T)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T,X> From<X> for Fun<T> where X: 'static + (Fn(T,T)->T) {
    fn from(x: X) -> Self {
        Fun(Box::new(x))
    }
}