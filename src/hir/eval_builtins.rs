use std::{
    mem,
    convert::AsRef,
    ops::DerefMut,
};
use super::*;

impl Expr {
    fn eval_builtins(&mut self) {
        match self {
            Expr::UnaryOp(op, a) => match a.as_ref() {
                Expr::Value(a) => if let Some(val) = op.eval_with(a) {
                    *self = Expr::Value(val);
                } else {},
                _ => {},
            },
            Expr::BinaryOp(op, a, b) => match (a.as_ref(), b.as_ref()) {
                (Expr::Value(a), Expr::Value(b)) => if let Some(val) = op.eval_with(a, b) {
                    *self = Expr::Value(val);
                } else {},
                _ => {},
            },
            _ => {},
        }
        self.visit_child_exprs(|expr| expr.eval_builtins());
    }
}

pub fn apply(prog: &mut Program) {
    for (_, expr) in &prog.globals {
        expr.borrow_mut().eval_builtins();
    }
}
