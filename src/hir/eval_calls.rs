use std::mem;
use super::*;

impl Expr {
    fn eval_calls(&mut self) {
        match self {
            Expr::CallExpr(callee, args) if args.len() > 0 => match callee.as_mut() {
                Expr::Value(Value::Func(param, body)) => {
                    let mut args_iter = args.iter_mut();
                    let next_arg = args_iter.next().unwrap();
                    body.replace_local(param, next_arg);
                    *callee = body.clone();
                    let new_args = args_iter.map(|expr| expr.clone()).collect();
                    *args = new_args;
                }
                _ => {},
            },
            Expr::CallExpr(callee, args) if args.len() == 0 => {
                *self = callee.as_ref().clone();
            },
            _ => {},
        }
        self.visit_child_exprs(|expr| expr.eval_calls());
    }
}

pub fn apply(prog: &mut Program) {
    for (_, expr) in &prog.globals {
        expr.borrow_mut().eval_calls();
    }
}
