use std::{
    slice,
    collections::HashMap,
    io::{self, prelude::*},
    env,
    fs,
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
        let s = s.trim_matches('\n');
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

    If,   Head,  Tail,
    Fuse, Pair,  Litr,
    Str,  Words, Input,
    Print,

    Add,  Neg,
    Mul, Div, Rem,
    Eq,
    Less, LessEq,

    Value(Value),
    Ident(String),
}

#[derive(Debug)]
enum Expr {
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Head(Box<Expr>),
    Tail(Box<Expr>),
    Fuse(Box<Expr>, Box<Expr>),
    Pair(Box<Expr>, Box<Expr>),
    Litr(Box<Expr>),
    Str(Box<Expr>),
    Words(Box<Expr>),
    Input(Box<Expr>),
    Print(Box<Expr>),

    Eq(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Rem(Box<Expr>, Box<Expr>),
    Less(Box<Expr>, Box<Expr>),
    LessEq(Box<Expr>, Box<Expr>),

    Value(Value),
    Call(String, Vec<Expr>),
    Local(usize),
}

#[derive(Debug)]
struct Func {
    args: Vec<String>,
    expr: Expr,
}

fn print(msg: String) {
    println!("{}", msg);
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
        Expr::Eq(x, y) => Value::Bool(eval(&x, funcs, args) == eval(&y, funcs, args)),
        Expr::Add(x, y) => match (eval(&x, funcs, args), eval(&y, funcs, args)) {
            (Value::Num(x), Value::Num(y)) => Value::Num(x + y),
            (Value::Str(x), Value::Str(y)) => Value::Str(x + &y),
            _ => Value::Null,
        },
        Expr::Neg(x) => match eval(&x, funcs, args) {
            Value::Num(x) => Value::Num(-x),
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
        Expr::Rem(x, y) => match (eval(&x, funcs, args), eval(&y, funcs, args)) {
            (Value::Num(x), Value::Num(y)) => Value::Num(x % y),
            _ => Value::Null,
        },
        Expr::Less(x, y) => match (eval(&x, funcs, args), eval(&y, funcs, args)) {
            (Value::Num(x), Value::Num(y)) => Value::Bool(x < y),
            (Value::Str(x), Value::Str(y)) => Value::Bool(x < y),
            _ => Value::Null,
        },
        Expr::LessEq(x, y) => match (eval(&x, funcs, args), eval(&y, funcs, args)) {
            (Value::Num(x), Value::Num(y)) => Value::Bool(x <= y),
            (Value::Str(x), Value::Str(y)) => Value::Bool(x <= y),
            _ => Value::Null,
        },
        Expr::Head(list) => match eval(&list, funcs, args) {
            Value::List(items) => items.first().cloned().unwrap_or(Value::Null),
            Value::Str(s) => s.get(0..1).map(|s| Value::Str(s.to_string())).unwrap_or(Value::Null),
            val => val,
        },
        Expr::Tail(list) => match eval(&list, funcs, args) {
            Value::List(items) => items.get(1..).map(|items| Value::List(items.iter().cloned().collect())).unwrap_or(Value::Null),
            Value::Str(s) => s.get(1..).map(|s| if s.len() == 0 { Value::Null } else { Value::Str(s.to_string()) }).unwrap_or(Value::Null),
            _ => Value::Null,
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
        Expr::Print(x) => {
            let val = eval(&x, funcs, args);
            print(val.clone().into_string());
            val
        },
        Expr::Str(x) => Value::Str(eval(&x, funcs, args).into_string()),
        Expr::Value(val) => val.clone(),
        Expr::Local(idx) => args.get(*idx).cloned().unwrap_or(Value::Null),
    }
}

fn parse_expr(tokens: &mut slice::Iter<Token>, args: &Vec<String>, func_defs: &HashMap<String, usize>) -> Result<Expr, Error> {
    Ok(match tokens.next().ok_or(Error::ExpectedToken)? {
        Token::If => Expr::If(
            Box::new(parse_expr(tokens, args, func_defs)?),
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

        Token::Eq => Expr::Eq(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),
        Token::Add => Expr::Add(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),
        Token::Neg => Expr::Neg(Box::new(parse_expr(tokens, args, func_defs)?)),
        Token::Mul => Expr::Mul(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),
        Token::Div => Expr::Div(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),
        Token::Rem => Expr::Rem(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),
        Token::Less => Expr::Less(
            Box::new(parse_expr(tokens, args, func_defs)?),
            Box::new(parse_expr(tokens, args, func_defs)?),
        ),
        Token::LessEq => Expr::LessEq(
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
    tokens
        .clone()
        .scan((None, &mut func_defs), |(state, funcs), tok| {
            match state {
                Some((name, n)) => match tok {
                    Token::Ident(i) => {
                        if *n == 0 {
                            *name = i.clone();
                        }
                        *n += 1;
                    },
                    Token::Is => {
                        funcs.insert(name.clone(), *n - 1);
                        *state = None;
                    },
                    _ => *n += 1,
                },
                None => match tok {
                    Token::Fn => *state = Some((String::new(), 0usize)),
                    _ => {},
                },
            }
            Some(tok)
        })
        .for_each(|_| ());

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
            "if" => Token::If,
            "__head" => Token::Head,
            "__tail" => Token::Tail,
            "__fuse" => Token::Fuse,
            "__pair" => Token::Pair,
            "__litr" => Token::Litr,
            "__str" => Token::Str,
            "__words" => Token::Words,
            "__input" => Token::Input,
            "__print" => Token::Print,
            "__eq" => Token::Eq,
            "__add" => Token::Add,
            "__neg" => Token::Neg,
            "__mul" => Token::Mul,
            "__div" => Token::Div,
            "__rem" => Token::Rem,
            "__less" => Token::Less,
            "__lesseq" => Token::LessEq,
            s => if let Some(v) = Value::from_str(s) {
                Token::Value(v)
            } else {
                Token::Ident(s.to_string())
            }
        })
        .collect::<Vec<_>>()
}

fn with_core(code: &str) -> String {
    String::from(include_str!("atto/core.at")) + code
}

fn prompt() {
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

    println!("Welcome to the Atto prompt.");
    println!("The core library is included by default.");

    let mut rl = Editor::<()>::new();
    while let Ok(line) = rl.readline(">> ") {
        rl.add_history_entry(line.as_ref());

        let _ = {
            let tokens = lex(&with_core(&line));

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

fn exec(fname: &str) {
    let mut code = String::new();
    match fs::File::open(fname) {
        Ok(mut file) => { file.read_to_string(&mut code).unwrap(); },
        Err(_) => println!("Could not open file '{}'", fname),
    }

    let _ = parse_funcs(lex(&with_core(&code)).iter()).map(|funcs| {
        if let Some(main) = funcs.get("main") {
            eval(&main.expr, &funcs, &mut vec![])
        } else {
            Value::Null
        }
    })
        .map_err(|err| print!("{:?}", err));
}

fn usage() {
    println!("Usage: atto [file]");
}

fn main() {
    match &env::args().nth(1) {
        None => prompt(),
        Some(arg) if env::args().count() == 2 => exec(arg),
        Some(_) => usage(),
    }
}
