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
    arity: usize,
    body: Expr,
}

impl Def {
    pub fn new(arity: usize, body: Expr) -> Self {
        Self {
            arity,
            body,
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Literal(Literal),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Let(Vec<(String, usize)>, Box<Expr>, Box<Expr>),
    Builtin(Box<Builtin>),
    Call(String, Vec<Expr>), // Includes things that have an arity of zero!
    Closure((String, usize), Box<Expr>),
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
    Head(Expr),
    Tail(Expr),
    Wrap(Expr),
    Cat(Expr, Expr),

    Input(Expr),
    Print(Expr, Expr),

    Add(Expr, Expr),
    Sub(Expr, Expr),
    Mul(Expr, Expr),
    Div(Expr, Expr),
    Rem(Expr, Expr),
    Eq(Expr, Expr),
    Less(Expr, Expr),
    LessEq(Expr, Expr),
}
