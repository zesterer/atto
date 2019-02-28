use std::{
    slice,
    collections::HashMap,
    io::{self, prelude::*},
};
use rustyline::Editor;

#[derive(Debug)]
enum Error {
    Expected(Token),
    ExpectedToken,
    Unexpected(Token),
    CannotFind(String),
}

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
    List(Vec<Value>),
    Null,
}

impl Value {
    pub fn from_str(s: &str) -> Option<Value> {
        let s = s.trim();
        if s == "null" {
            Some(Value::Null)
        } else if s == "true" {
            Some(Value::Bool(true))
        } else if s == "false" {
            Some(Value::Bool(false))
        } else if let Ok(x) = s.parse() {
            Some(Value::Num(x))
        } else if let Ok(b) = s.parse() {
            Some(Value::Bool(b))
        } else if s.chars().next() == Some('"' /*"*/) {
            Some(Value::Str(s[1..].to_string()))
        } else {
            None
        }
    }

    pub fn into_string(self) -> String {
        match self {
            Value::Num(x) => format!("{}", x),
            Value::Str(s) => s,
            Value::Bool(b) => format!("{}", b),
            Value::List(l) => {
                let mut s = String::from("[");
                for i in 0..l.len() {
                    if i != 0 {
                        s += ", ";
                    }
                    s += &format!("{}", l[i].clone().into_string());
                }
                s += "]";
                s
            },
            Value::Null => "null".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
enum Token {
    Fn, Is,

    In,    If,    Head, Tail,
    Fuse,  Pair,  Litr, Str,
    Words, Input, Print,

    Not, Eq, Add, Sub, Mul, Div,

    Value(Value),
    Ident(String),
}

#[derive(Debug)]
enum Expr {
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    In(Box<Expr>, Box<Expr>),
    Head(Box<Expr>),
    Tail(Box<Expr>),
    Fuse(Box<Expr>, Box<Expr>),
    Pair(Box<Expr>, Box<Expr>),
    Litr(Box<Expr>),
    Str(Box<Expr>),
    Words(Box<Expr>),
    Input(Box<Expr>),
    Print(Box<Expr>),

    Not(Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),

    Value(Value),
    Call(String, Vec<Expr>),
    Local(usize),
}

#[derive(Debug)]
struct Func {
    args: Vec<String>,
    expr: Expr,
}

fn print(msg: String) -> Value {
    println!("{}", msg);
    Value::Null
}

fn input(msg: String) -> Value {
    print!("{}", msg);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input = input.replace('\n', "");
    Value::Str(input)
}

fn eval(expr: &Expr, funcs: &HashMap<String, Func>, args: &Vec<Value>) -> Value {
    match expr {
        Expr::If(pred, good, bad) => if eval(&pred, funcs, args) == Value::Bool(true) {
            eval(&good, funcs, args)
        } else {
            eval(&bad, funcs, args)
        },
        Expr::In(expr, list) => match eval(&list, funcs, args) {
            Value::List(items) => Value::Bool(items.contains(&eval(&expr, funcs, args))),
            _ => Value::Null
        },
        Expr::Not(x) => match eval(&x, funcs, args) {
            Value::Bool(b) => Value::Bool(!b),
            _ => Value::Null,
        },
        Expr::Eq(x, y) => Value::Bool(eval(&x, funcs, args) == eval(&y, funcs, args)),
        Expr::Add(x, y) => match (eval(&x, funcs, args), eval(&y, funcs, args)) {
            (Value::Num(x), Value::Num(y)) => Value::Num(x + y),
            (Value::Str(x), Value::Str(y)) => Value::Str(x + &y),
            _ => Value::Null,
        },
        Expr::Sub(x, y) => match (eval(&x, funcs, args), eval(&y, funcs, args)) {
            (Value::Num(x), Value::Num(y)) => Value::Num(x - y),
            _ => Value::Null,
        },
        Expr::Mul(x, y) => match (eval(&x, funcs, args), eval(&y, funcs, args)) {
            (Value::Num(x), Value::Num(y)) => Value::Num(x * y),
            _ => Value::Null,
        },
        Expr::Div(x, y) => match (eval(&x, funcs, args), eval(&y, funcs, args)) {
            (Value::Num(x), Value::Num(y)) => Value::Num(x / y),
            _ => Value::Null,
        },
        Expr::Head(list) => if let Value::List(items) = eval(&list, funcs, args) {
            items.first().cloned().unwrap_or(Value::Null)
        } else {
            Value::Null
        },
        Expr::Tail(list) => if let Value::List(items) = eval(&list, funcs, args) {
            items.get(1..).map(|items| Value::List(items.iter().cloned().collect())).unwrap_or(Value::Null)
        } else {
            Value::Null
        },
        Expr::Fuse(x, y) => match (eval(&x, funcs, args), eval(&y, funcs, args)) {
            (Value::List(mut x), Value::List(mut y)) => Value::List({ x.append(&mut y); x }),
            (Value::List(mut x), y) => Value::List({ x.push(y); x }),
            (x, Value::List(mut y)) => Value::List({ let mut v = vec![x]; v.append(&mut y); v }),
            (x, y) => Value::List(vec![x, y]),
        },
        Expr::Pair(x, y) => Value::List(vec![eval(&x, funcs, args), eval(&y, funcs, args)]),
        Expr::Call(f, params) => if let Some(f) = funcs.get(f) {
            eval(&f.expr, funcs, &params.iter().map(|p| eval(&p, funcs, args)).collect())
        } else {
            Value::Null
        },
        Expr::Words(x) => if let Value::Str(s) = eval(&x, funcs, args) {
            Value::List(words(&s).into_iter().map(|s| Value::Str(s)).collect())
        } else {
            Value::Null
        },
        Expr::Litr(x) => if let Value::Str(s) = eval(&x, funcs, args) {
            Value::from_str(&s).unwrap_or(Value::Null)
        } else {
            Value::Null
        },
        Expr::Input(x) => input(eval(&x, funcs, args).into_string()),
        Expr::Print(x) => print(eval(&x, funcs, args).into_string()),
        Expr::Str(x) => Value::Str(eval(&x, funcs, args).into_string()),
        Expr::Value(val) => val.clone(),
        Expr::Local(idx) => args[*idx].clone(),
    }
}

fn parse_expr(tokens: &mut slice::Iter<Token>, args: &Vec<String>, func_defs: &HashMap<String, usize>) -> Result<Expr, Error> {
    Ok(match tokens.next().ok_or(Error::ExpectedToken)? {
        Token::If => Expr::If(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),
        Token::In => Expr::In(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),
        Token::Head => Expr::Head(Box::new(parse_expr(tokens, args, func_defs)?)),
        Token::Tail => Expr::Tail(Box::new(parse_expr(tokens, args, func_defs)?)),
        Token::Fuse => Expr::Fuse(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),
        Token::Pair => Expr::Pair(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),
        Token::Litr => Expr::Litr(Box::new(parse_expr(tokens, args, func_defs)?)),
        Token::Str => Expr::Str(Box::new(parse_expr(tokens, args, func_defs)?)),
        Token::Words => Expr::Words(Box::new(parse_expr(tokens, args, func_defs)?)),
        Token::Input => Expr::Input(Box::new(parse_expr(tokens, args, func_defs)?)),
        Token::Print => Expr::Print(Box::new(parse_expr(tokens, args, func_defs)?)),
        Token::Value(v) => Expr::Value(v.clone()),

        Token::Not => Expr::Not(Box::new(parse_expr(tokens, args, func_defs)?)),
        Token::Eq => Expr::Eq(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),
        Token::Add => Expr::Add(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),
        Token::Sub => Expr::Sub(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),
        Token::Mul => Expr::Mul(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),
        Token::Div => Expr::Div(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),

        Token::Ident(i) => {
            if let Some((idx, _)) = args
                .iter()
                .enumerate()
                .find(|(_, arg)| &i == arg)
            {
                Expr::Local(idx)
            } else if let Some(f_args) = func_defs.get(i.as_str()) {
                let mut params = vec![];
                for _ in 0..*f_args {
                    params.push(parse_expr(tokens, args, func_defs)?);
                }
                Expr::Call(i.clone(), params)
            } else {
                return Err(Error::CannotFind(i.clone()));
            }
        },
        t => return Err(Error::Unexpected(t.clone())),
    })
}

fn parse_funcs(mut tokens: slice::Iter<Token>) -> Result<HashMap<String, Func>, Error> {
    let mut funcs = HashMap::new();
    let mut func_defs = HashMap::new();
    loop {
        match tokens.next() {
            Some(Token::Fn) => {},
            _ => return Ok(funcs),
        }

        let name = match tokens.next() {
            Some(Token::Ident(s)) => s.clone(),
            _ => return Err(Error::Expected(Token::Fn)),
        };

        let mut args = vec![];
        loop {
            match tokens.next() {
                Some(Token::Ident(s)) => args.push(s.clone()),
                Some(Token::Is) => break,
                _ => return Err(Error::Expected(Token::Is)),
            }
        }

        func_defs.insert(name.clone(), args.len());

        let expr = parse_expr(&mut tokens, &args, &func_defs)?;

        funcs.insert(name, Func {
            args,
            expr,
        });
    }
}

fn words(s: &str) -> Vec<String> {
    s
        .chars()
        .chain(Some(' '))
        .scan((false, String::new()), |(in_str, buf), c| {
            match c {
                '"' /*"*/ => if *in_str {
                    *in_str = false;
                } else {
                    buf.push('"' /*"*/);
                    *in_str = true;
                },
                c if c.is_whitespace() && !*in_str => {
                    let s = buf.clone();
                    buf.clear();
                    return Some(s);
                },
                c => buf.push(c),
            }
            Some("".to_string())
        })
        .filter(|s| s.len() > 0)
        .collect()
}

fn lex(code: &str) -> Vec<Token> {
    words(code)
        .into_iter()
        .map(|s| match s.as_str() {
            "fn" => Token::Fn,
            "is" => Token::Is,
            "in" => Token::In,
            "if" => Token::If,
            "head" => Token::Head,
            "tail" => Token::Tail,
            "fuse" => Token::Fuse,
            "pair" => Token::Pair,
            "litr" => Token::Litr,
            "str" => Token::Str,
            "words" => Token::Words,
            "input" => Token::Input,
            "print" => Token::Print,
            "=" => Token::Eq,
            "+" => Token::Add,
            "-" => Token::Sub,
            "*" => Token::Mul,
            "/" => Token::Div,
            "!" => Token::Not,
            s => if let Some(v) = Value::from_str(s) {
                Token::Value(v)
            } else {
                Token::Ident(s.to_string())
            }
        })
        .collect::<Vec<_>>()
}

fn main() {
    /*
    let code = include_str!("eval.at");

    let tokens = lex(code);

    let funcs = parse_funcs(tokens.iter()).unwrap();

    let example = r#"
    + 5 * 2 3
    "#;

    let result = eval(
        &funcs.get("main").unwrap().expr,
        &funcs,
        &mut vec![
            Value::Str(example.to_string())
        ],
    );

    println!("Result: {:?}", result);
    */

    let mut rl = Editor::<()>::new();
    while let Ok(line) = rl.readline(">> ") {
        rl.add_history_entry(line.as_ref());

        let _ = {
            let tokens = lex(&line);

            parse_funcs(tokens.iter()).map(|funcs| {
                if let Some(main) = funcs.get("main") {
                    eval(&main.expr, &funcs, &mut vec![])
                } else {
                    Value::Null
                }
            })
            .and_then(|_| parse_expr(&mut tokens.iter(), &vec![], &HashMap::new()).map(|expr| {
                eval(&expr, &HashMap::new(), &mut vec![])
            }))
        }
            .map(|val| println!("{}", val.into_string()))
            .map_err(|err| print!("{:?}", err));
    }
}
