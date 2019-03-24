use std::{
    rc::Rc,
    collections::HashMap,
    fmt::Debug,
};
use crate::exec::value::Value;

pub trait CustomFunc: Debug {
    fn call(&self, _args: &[Value]) -> Value;
}

#[derive(Clone, Debug)]
pub struct Prog {
    globals: HashMap<String, (Rc<String>, Rc<Func>)>,
    entry: String,
}

#[derive(Clone, Debug)]
pub struct Func {
    args: usize,
    body: Body,
}

#[derive(Clone, Debug)]
pub enum Body {
    Native(Expr),
    Custom(Rc<dyn CustomFunc>),
}

#[derive(Clone, Debug)]
pub enum Expr {
    Value(Value),
    Push {
        val: Var,
        then: Rc<Expr>,
    },
    Call {
        func: Rc<String>,
        args: Vec<Var>,
    },
    Return,
    ReplaceFrame {
        args: Vec<Var>,
    },
}

#[derive(Clone, Debug)]
pub enum Var {
    Expr(Rc<Expr>),
    MoveStack {
        offset: usize,
    },
    CloneStack {
        offset: usize,
    },
}
