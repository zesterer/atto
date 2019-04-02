use std::{
    rc::Rc,
    cell::Cell,
    fmt::Debug,
};
use crate::ir::Func;

pub trait CustomFunc: Debug {
    fn arity(&self) -> (usize, usize);
    fn call(&self, _args: &[Value]) -> Value;
}

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str {
        offset: usize,
        buf: Rc<Vec<char>>,
    },
    List {
        offset: usize,
        buf: Rc<Vec<Value>>,
    },
    Func(Rc<Func>),
    Custom(Rc<dyn CustomFunc>),
    Universe(Cell<bool>),
}
