use std::{
    io,
    rc::Rc,
    collections::HashMap,
    cell::Cell,
    sync::atomic::{
        Ordering,
        AtomicUsize,
    },
};
use crate::{
    Error,
    parse::{
        self,
        ast::{
            Program,
            Expr,
            Decl,
            Literal,
            Builtin,
        },
    },
};

#[derive(Clone, Debug)]
pub enum Value<'a> {
    Null,
    Bool(bool),
    Num(f64),
    Char(char),
    List {
        offset: usize,
        buf: Rc<Vec<Value<'a>>>,
    },
    Func(HashMap<&'a str, Value<'a>>, &'a Decl, &'a Expr),
    Universe(usize),
}

static NEXT_UNIVERSE: AtomicUsize = AtomicUsize::new(0);

impl<'a> Value<'a> {
    fn to_str(&self) -> String {
        match self {
            Value::Null => format!("null"),
            Value::Bool(a) => format!("{}", a),
            Value::Num(a) => format!("{}", a),
            Value::Char(a) => format!("{}", a),
            Value::List { offset, buf } => {
                if buf[*offset..].len() == 0 {
                    format!("[]")
                } else if let Some(s) = buf[*offset..].iter().try_fold(String::new(), |mut s, v| {
                    if let Value::Char(c) = v {
                        s.push(*c);
                        Some(s)
                    } else {
                        None
                    }
                }) {
                    format!("{}", s)
                } else {
                    format!("[{}]", buf[*offset..].iter().map(|v| v.to_str()).collect::<Vec<_>>().join(", "))
                }
            },
            Value::Func(_, _, _) => format!("<func>"),
            Value::Universe(_) => format!("<universe>"),
        }
    }

    fn input(self) -> Self {
        match self {
            Value::Universe(a) if a == NEXT_UNIVERSE.fetch_add(1, Ordering::Relaxed) => {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();

                Value::List {
                    offset: 0,
                    buf: Rc::new(vec![
                        Value::Universe((a + 1).into()),
                        Value::List {
                            offset: 0,
                            buf: Rc::new(input.chars().map(|c| Value::Char(c)).collect()),
                        },
                    ]),
                }
            },
            _ => panic!("Invalid universe value!"),
        }
    }

    fn print(self, other: Self) -> Self {
        match self {
            Value::Universe(a) if a == NEXT_UNIVERSE.fetch_add(1, Ordering::Relaxed) => {
                println!("{}", other.to_str());
                Value::Universe((a + 1).into())
            },
            _ => panic!("Invalid universe value!"),
        }
    }

    fn head(self) -> Self {
        match self {
            Value::List { offset, buf } => if let Some(head) = buf.get(offset) {
                head.clone()
            } else {
                Value::Null
            },
            a => a,
        }
    }

    fn tail(self) -> Self {
        match self {
            Value::List { offset, buf } => Value::List {
                offset: (offset + 1).min(buf.len()),
                buf: buf.clone(),
            },
            a => Value::Null,
        }
    }

    fn wrap(self) -> Self {
        Value::List {
            offset: 0,
            buf: Rc::new(vec![self]),
        }
    }

    fn cat(self, other: Self) -> Self {
        match (self, other) {
            (Value::List {
                offset: offset_a, buf: buf_a
            }, Value::List {
                offset: offset_b, buf: buf_b
            }) => {
                let mut v = Vec::from(&buf_a[offset_a..]);
                v.extend_from_slice(&buf_b[offset_b..]);
                Value::List {
                    offset: 0,
                    buf: Rc::new(v),
                }
            },
            (Value::List {
                offset: offset_a, buf: buf_a
            }, b) => {
                let mut v = Vec::from(&buf_a[offset_a..]);
                v.push(b);
                Value::List {
                    offset: 0,
                    buf: Rc::new(v),
                }
            },
            (a, Value::List {
                offset: offset_b, buf: buf_b
            }) => {
                let mut v = vec![a];
                v.extend_from_slice(&buf_b[offset_b..]);
                Value::List {
                    offset: 0,
                    buf: Rc::new(v),
                }
            },
            _ => Value::Null,
        }
    }

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Value::Num(a + b),
            _ => unimplemented!(),
        }
    }

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Value::Num(a - b),
            _ => unimplemented!(),
        }
    }

    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Value::Num(a * b),
            _ => unimplemented!(),
        }
    }

    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Value::Num(a / b),
            _ => unimplemented!(),
        }
    }

    fn rem(self, other: Self) -> Self {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Value::Num(a % b),
            _ => unimplemented!(),
        }
    }

    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Num(a), Value::Num(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::List {
                offset: offset_a, buf: buf_a
            }, Value::List {
                offset: offset_b, buf: buf_b
            }) => {
                let len_a = (buf_a.len() - offset_a);
                let len_b = (buf_b.len() - offset_b);
                if len_a == 0 && len_b == 0 {
                    true
                } else {
                    len_a == len_b &&
                    !buf_a.iter().zip(buf_b.iter()).any(|(a, b)| !a.eq(b))
                }
            },
            _ => false,
        }
    }

    fn less(self, other: Self) -> bool {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => a < b,
            _ => unimplemented!(),
        }
    }

    fn lesseq(self, other: Self) -> bool {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => a <= b,
            _ => unimplemented!(),
        }
    }

    fn floor(self) -> Value<'a> {
        match self {
            Value::Num(a) => Value::Num(a.floor()),
            _ => unimplemented!(),
        }
    }

    fn ceil(self) -> Value<'a> {
        match self {
            Value::Num(a) => Value::Num(a.ceil()),
            _ => unimplemented!(),
        }
    }

    fn call(&self, prog: &'a Program, args: &[Value<'a>]) -> Self {
        if let Some(arg) = args.get(0) {
            match self {
                Value::Func(locals, decl, body) => {
                    let mut locals = locals.clone();

                    match decl {
                        Decl::Single(name) => { locals.insert(name, arg.clone()); },
                        Decl::Destructure(names) => match arg {
                            Value::List { offset, buf } => if buf[*offset..].len() != names.len() {
                                panic!("Cannot destructure list of incorrect length");
                            } else {
                                for (name, val) in names.iter().zip(buf[*offset..].into_iter()) {
                                    locals.insert(name, val.clone());
                                }
                            },
                            _ => panic!("Cannot destructure non-list!"),
                        },
                    }

                    eval(body, prog, &args[1..], &locals)
                },
                _ => panic!("Too many arguments!"),
            }
        } else {
            self.clone()
        }
    }
}

pub fn exec(code: &str) -> Result<(), Vec<Error>> {
    let mut src = String::from(include_str!("../../atto/core.at"));
    src += code;
    run_prog(&parse::code(&src)?).map_err(|err| vec![err])?;
    Ok(())
}

fn run_prog(prog: &Program) -> Result<Value, Error> {
    if let Some(main) = prog.globals.get("main") {
        Ok(eval(main, prog, &vec![Value::Universe(NEXT_UNIVERSE.load(Ordering::Relaxed))], &HashMap::new()))
    } else {
        Err(Error::no_main())
    }
}

fn eval<'a>(expr: &'a Expr, prog: &'a Program, args: &[Value<'a>], locals: &HashMap<&'a str, Value<'a>>) -> Value<'a> {
    //println!("Evaluating... {:?}", expr);
    match expr {
        Expr::Literal(lit) => match lit {
            Literal::Num(x) => Value::Num(*x),
            Literal::Str(s) => if s.len() == 1 {
                Value::Char(s.chars().next().unwrap())
            } else {
                Value::List {
                    offset: 0,
                    buf: Rc::new(s.chars().map(|c| Value::Char(c)).collect()),
                }
            },
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Null => Value::Null,
        },
        Expr::If(predicate, true_expr, false_expr) => if let Value::Bool(true) = eval(predicate, prog, args, locals) {
            eval(true_expr, prog, args, locals)
        } else {
            eval(false_expr, prog, args, locals)
        },
        Expr::Let(decl, expr, then) => {
            let val = eval(expr, prog, args, locals);
            let mut locals = locals.clone();

            match decl {
                Decl::Single(name) => { locals.insert(name, val); },
                Decl::Destructure(names) => match val {
                    Value::List { offset, buf } => if buf[offset..].len() != names.len() {
                        panic!("Cannot destructure list of incorrect length");
                    } else {
                        for (name, val) in names.iter().zip(buf[offset..].into_iter()) {
                            locals.insert(name, val.clone());
                        }
                    },
                    _ => panic!("Cannot destructure non-list!"),
                },
            }

            eval(then, prog, args, &locals)
        },
        Expr::Builtin(builtin) => match builtin.as_ref() {
            Builtin::Print(a, b) => eval(&a, prog, args, locals).print(eval(&b, prog, args, locals)),
            Builtin::Input(a) => eval(&a, prog, args, locals).input(),
            Builtin::Debug(a) => {
                let val = eval(&a, prog, args, locals);
                println!("{:?}", val);
                val
            },

            Builtin::Head(a) => eval(&a, prog, args, locals).head(),
            Builtin::Tail(a) => eval(&a, prog, args, locals).tail(),
            Builtin::Wrap(a) => eval(&a, prog, args, locals).wrap(),
            Builtin::Cat(a, b) => eval(&a, prog, args, locals).cat(eval(&b, prog, args, locals)),

            Builtin::Add(a, b) => eval(&a, prog, args, locals).add(eval(&b, prog, args, locals)),
            Builtin::Sub(a, b) => eval(&a, prog, args, locals).sub(eval(&b, prog, args, locals)),
            Builtin::Mul(a, b) => eval(&a, prog, args, locals).mul(eval(&b, prog, args, locals)),
            Builtin::Div(a, b) => eval(&a, prog, args, locals).div(eval(&b, prog, args, locals)),
            Builtin::Rem(a, b) => eval(&a, prog, args, locals).rem(eval(&b, prog, args, locals)),

            Builtin::Eq(a, b) => Value::Bool(eval(&a, prog, args, locals).eq(&eval(&b, prog, args, locals))),
            Builtin::Less(a, b) => Value::Bool(eval(&a, prog, args, locals).less(eval(&b, prog, args, locals))),
            Builtin::LessEq(a, b) => Value::Bool(eval(&a, prog, args, locals).lesseq(eval(&b, prog, args, locals))),
            Builtin::Floor(a) => eval(&a, prog, args, locals).floor(),
            Builtin::Ceil(a) => eval(&a, prog, args, locals).ceil(),
            _ => unimplemented!(),
        },
        Expr::Call(name, call_args) => {
            let mut call_args: Vec<_> = call_args
                .iter()
                .map(|expr| eval(expr, prog, args, locals))
                .collect();
            call_args.extend_from_slice(args);

            if let Some(local) = locals.get(name.as_str()) {
                local.call(prog, &call_args)
            } else if let Some(global) = prog.globals.get(name) {
                eval(global, prog, &call_args, &HashMap::new())
            } else {
                panic!("Could not find item '{}'", name);
            }
        },
        Expr::Closure(decl, body) => match args.get(0) {
            Some(arg) => {
                let mut locals = locals.clone();

                match decl {
                    Decl::Single(name) => { locals.insert(name, arg.clone()); },
                    Decl::Destructure(names) => match arg {
                        Value::List { offset, buf } => if buf[*offset..].len() != names.len() {
                            panic!("Cannot destructure list of incorrect length");
                        } else {
                            for (name, val) in names.iter().zip(buf[*offset..].into_iter()) {
                                locals.insert(name, val.clone());
                            }
                        },
                        _ => panic!("Cannot destructure non-list!"),
                    },
                }

                eval(body, prog, &args[1..], &locals)
            },
            None => Value::Func(locals.clone(), decl, body),
        },
        _ => unimplemented!(),
    }
}
