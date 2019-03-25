use std::{
    rc::Rc,
    collections::HashMap,
};
use crate::exec::value::Value;

#[derive(Clone, Debug)]
pub struct Program {
    globals: HashMap<String, (Rc<String>, Func)>,
    entry: Rc<String>,
}

#[derive(Clone, Debug)]
pub struct Func {
    args: Vec<Rc<String>>,
    env: Vec<Rc<String>>,
    body: Expr,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Value(Value),
    Builtin(Builtin),
    Call {
        name: Rc<String>,
        args: Vec<Expr>,
    },
    If {
        predicate: Box<Expr>,
        true_block: Box<Expr>,
        false_block: Box<Expr>,
    },
    Let {
        names: Rc<String>,
        expr: Box<Expr>,
    },
    Many {
        exprs: Vec<Expr>,
    },
}

#[derive(Clone, Debug)]
pub enum Builtin {
    Head(Box<Expr>),
    Tail(Box<Expr>),
    Wrap(Box<Expr>),
    Cat(Box<Expr>, Box<Expr>),

    Input(Box<Expr>),
    Print(Box<Expr>, Box<Expr>),

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Rem(Box<Expr>, Box<Expr>),
    Less(Box<Expr>, Box<Expr>),
    LessEq(Box<Expr>, Box<Expr>),
}
