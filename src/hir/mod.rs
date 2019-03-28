mod eval_builtins;
mod eval_calls;
mod inline_globals;

use std::{
    mem,
    rc::{Weak, Rc},
    cell::RefCell,
    collections::HashMap,
};

#[derive(Debug)]
pub struct Program {
    pub globals: HashMap<String, RefCell<Expr>>,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Value(Value),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    CallGlobal(String, Vec<Expr>),
    CallLocal(String, Vec<Expr>),
    CallExpr(Box<Expr>, Vec<Expr>),
}

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Char(char),
    List(Vec<Value>),
    Func(String, Box<Expr>),
    Universe,
}

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Head,
    Tail,
    Wrap,
    Input,
    Debug,
    Floor,
    Ceil,
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Cat,
    Print,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Less,
    LessEq,
}

impl Program {
    pub fn optimise_once(mut self) -> Self {
        inline_globals::apply(&mut self);
        eval_calls::apply(&mut self);
        eval_builtins::apply(&mut self);
        self
    }
}

impl Expr {
    fn visit_child_exprs<F: Fn(&mut Expr)>(&mut self, f: F) {
        match self {
            Expr::Value(Value::Func(_, a)) => { f(a); },
            Expr::If(a, b, c) => { f(a); f(b); f(c); },
            Expr::Let(_, a, b) => { f(a); f(b); },
            Expr::UnaryOp(_, a) => { f(a); },
            Expr::BinaryOp(_, a, b) => { f(a); f(b); },
            Expr::CallGlobal(_, exprs) => for expr in exprs.iter_mut() { f(expr); },
            Expr::CallLocal(_, exprs) => for expr in exprs.iter_mut() { f(expr); },
            Expr::CallExpr(expr, exprs) => {
                f(expr);
                for expr in exprs.iter_mut() {
                    f(expr);
                }
            },
            _ => {},
        }
    }

    fn replace_local(&mut self, local: &str, new_local: &Expr) {
        if match self {
            Expr::Let(name, _, _) if name == local => false, // TODO: Replace first expr!
            Expr::Value(Value::Func(param, _)) if param == local => false,
            Expr::CallLocal(name, args) if name == local => {
                let mut tmp_args = Vec::new();
                mem::swap(args, &mut tmp_args);
                *self = Expr::CallExpr(Box::new(new_local.clone()), tmp_args);
                true
            },
            _ => true,
        } {
            self.visit_child_exprs(|expr| expr.replace_local(local, new_local));
        }
    }
}

impl UnaryOp {
    pub fn eval_with(&self, a: &Value) -> Option<Value> {
        match (self, a) {
            (UnaryOp::Floor, Value::Num(a)) => Some(Value::Num(a.floor())),
            (UnaryOp::Ceil, Value::Num(a)) => Some(Value::Num(a.ceil())),
            _ => None,
        }
    }
}

impl BinaryOp {
    pub fn eval_with(&self, a: &Value, b: &Value) -> Option<Value> {
        match (self, a, b) {
            (BinaryOp::Add, Value::Num(a), Value::Num(b)) => Some(Value::Num(*a + *b)),
            _ => None,
        }
    }
}
