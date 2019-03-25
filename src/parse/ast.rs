use std::collections::HashMap;

#[derive(Debug)]
pub struct Program {
    pub globals: HashMap<String, Def>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Def {
    arity: (usize, usize),
    body: Vec<Expr>,
}

impl Def {
    pub fn new(arity: (usize, usize), body: Vec<Expr>) -> Self {
        Self {
            arity,
            body,
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Literal(Literal),
    If(Vec<Expr>),
    Let(Vec<(String, (usize, usize))>, Box<Expr>, Box<Expr>),
    Builtin(Builtin),
    Call(String, Vec<Expr>), // Includes things that have an arity of zero!
    Closure((String, (usize, usize)), Vec<Expr>),
    Many(Vec<Expr>),
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
    Head(Vec<Expr>),
    Tail(Vec<Expr>),
    Wrap(Vec<Expr>),
    Cat(Vec<Expr>),

    Input(Vec<Expr>),
    Print(Vec<Expr>),

    Add(Vec<Expr>),
    Sub(Vec<Expr>),
    Mul(Vec<Expr>),
    Div(Vec<Expr>),
    Rem(Vec<Expr>),
    Eq(Vec<Expr>),
    Less(Vec<Expr>),
    LessEq(Vec<Expr>),
}
