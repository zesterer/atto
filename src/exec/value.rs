use std::rc::Rc;
use crate::ir::Func;

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
}
