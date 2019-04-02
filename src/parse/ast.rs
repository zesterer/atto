use std::{
    cell::RefCell,
    collections::HashMap,
};
use crate::hir;

#[derive(Debug)]
pub struct Program {
    pub globals: HashMap<String, Expr>,
}

#[derive(Debug)]
pub enum Expr {
    Literal(Literal),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Let(Decl, Box<Expr>, Box<Expr>),
    Builtin(Box<Builtin>),
    Call(String, Vec<Expr>), // Includes things that have an arity of zero!
    Closure(Decl, Box<Expr>),
}

#[derive(Debug)]
pub enum Decl {
    Single(String),
    Destructure(Vec<String>),
}

#[derive(Debug)]
pub enum Literal {
    Num(f64),
    Str(String),
    Bool(bool),
    Null,
}

#[derive(Debug)]
pub enum Builtin {
    Head(Expr), Tail(Expr),
    Wrap(Expr), Cat(Expr, Expr),

    Input(Expr), Print(Expr, Expr), Debug(Expr),

    Add(Expr, Expr), Sub(Expr, Expr),
    Mul(Expr, Expr), Div(Expr, Expr), Rem(Expr, Expr),
    Eq(Expr, Expr), Less(Expr, Expr), LessEq(Expr, Expr),
    Floor(Expr), Ceil(Expr),
}

impl Program {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
        }
    }

    pub fn to_hir(&self) -> hir::Program {
        let global_names = self.globals
            .iter()
            .map(|(name, _)| (name.as_str(), true))
            .collect();
        hir::Program {
            globals: self.globals
                .iter()
                .map(|(name, expr)|
                    (name.clone(), RefCell::new(expr.to_hir(&global_names))))
                .collect(),
        }
    }
}

impl Expr {
    pub fn to_hir(&self, names: &Vec<(&str, bool)>) -> hir::Expr {
        match self {
            Expr::Literal(lit) => lit.to_hir(),
            Expr::If(a, b, c) => hir::Expr::If(
                Box::new(a.to_hir(&names)),
                Box::new(b.to_hir(&names)),
                Box::new(c.to_hir(&names)),
            ),
            Expr::Let(decl, expr, body) => {
                let expr_hir = expr.to_hir(names);
                let mut names = names.clone();
                names.append(&mut decl.get_idents().into_iter().map(|ident| (ident, false)).collect());
                hir::Expr::Let(decl.to_hir(), Box::new(expr_hir), Box::new(body.to_hir(&names)))
            },
            Expr::Builtin(builtin) => builtin.to_hir(names),
            Expr::Call(name, exprs) => {
                if names.iter().find(|(n, _)| n == name).unwrap().1 {
                    hir::Expr::CallGlobal(name.clone(), exprs.iter().map(|expr| expr.to_hir(names)).collect())
                } else {
                    hir::Expr::CallLocal(name.clone(), exprs.iter().map(|expr| expr.to_hir(names)).collect())
                }
            },
            Expr::Closure(decl, expr) => {
                let mut names = names.clone();
                names.append(&mut decl.get_idents().into_iter().map(|ident| (ident, false)).collect());
                hir::Expr::Value(hir::Value::Func(decl.to_hir(), Box::new(expr.to_hir(&names))))
            },
            expr => unimplemented!("{:?}", expr),
        }
    }
}

impl Decl {
    pub fn to_hir(&self) -> hir::Decl {
        match self {
            Decl::Single(ident) => hir::Decl::Single(ident.clone()),
            Decl::Destructure(idents) => hir::Decl::Destructure(idents.iter().cloned().collect()),
        }
    }

    pub fn get_idents(&self) -> Vec<&str> {
        match self {
            Decl::Single(one) => vec![one],
            Decl::Destructure(many) => many.iter().map(|ident| ident.as_str()).collect(),
        }
    }
}

impl Literal {
    pub fn to_hir(&self) -> hir::Expr {
        match self {
            Literal::Num(n) =>
                hir::Expr::Value(hir::Value::Num(*n)),
            Literal::Str(s) =>
                hir::Expr::Value(hir::Value::List(s.chars().map(|c| hir::Value::Char(c)).collect())),
            Literal::Bool(b) =>
                hir::Expr::Value(hir::Value::Bool(*b)),
            Literal::Null =>
                hir::Expr::Value(hir::Value::Null),
        }
    }
}

impl Builtin {
    pub fn to_hir(&self, names: &Vec<(&str, bool)>) -> hir::Expr {
        match self {
            Builtin::Add(x, y) => hir::Expr::BinaryOp(hir::BinaryOp::Add, Box::new(x.to_hir(names)), Box::new(y.to_hir(names))),
            Builtin::Sub(x, y) => hir::Expr::BinaryOp(hir::BinaryOp::Sub, Box::new(x.to_hir(names)), Box::new(y.to_hir(names))),
            Builtin::Print(x, y) => hir::Expr::BinaryOp(hir::BinaryOp::Print, Box::new(x.to_hir(names)), Box::new(y.to_hir(names))),
            builtin => unimplemented!("{:?}", builtin),
        }
    }
}



