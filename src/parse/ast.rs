use std::collections::HashMap;

#[derive(Debug)]
pub struct Program {
    pub globals: HashMap<String, Expr>,
}

#[derive(Debug)]
pub enum Expr {
    Literal(Literal),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    LetDestructure(Vec<String>, Box<Expr>, Box<Expr>),
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
    Head(Expr), Tail(Expr),
    Wrap(Expr), Cat(Expr, Expr),

    Input(Expr), Print(Expr, Expr),

    Add(Expr, Expr), Sub(Expr, Expr),
    Mul(Expr, Expr), Div(Expr, Expr), Rem(Expr, Expr),
    Eq(Expr, Expr), Less(Expr, Expr), LessEq(Expr, Expr),
}

impl Program {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
        }
    }
}
