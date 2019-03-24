use std::collections::HashMap;

pub struct Prog {
    globals: HashMap<String, Func>,
}

pub struct Func {
    args: Vec<(String, usize)>,
    body: Expr,
}

pub enum Expr {
    Literal(Literal),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Let((String, usize), Box<Expr>, Box<Expr>),
    Builtin(Builtin),
    Call(String, Vec<Expr>), // Includes things that have an arity of zero!
}

pub enum Literal {
    Num(f64),
    Str(String),
    Bool(bool),
    Null,
}

pub enum Builtin {
    Head(Box<Expr>),
    Tail(Box<Expr>),
    Wrap(Box<Expr>),
    Cat(Box<Expr>, Box<Expr>),

    Input(Box<Expr>),
    Print(Box<Expr>),

    Eq(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Rem(Box<Expr>, Box<Expr>),
    Less(Box<Expr>, Box<Expr>),
    LessEq(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}
