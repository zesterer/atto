use std::mem;
use super::*;

impl Expr {
    fn inline_globals_except(&mut self, prog: &Program, except_global: &str) {
        match self {
            Expr::CallGlobal(name, args) if name != except_global => { // Don't inline itself!
                let mut tmp_args = Vec::new();
                mem::swap(args, &mut tmp_args);

                let global_expr = prog.globals.get(name).unwrap().borrow().clone();
                *self = Expr::CallExpr(Box::new(global_expr), tmp_args);
            },
            _ => {},
        }
        self.visit_child_exprs(|expr| expr.inline_globals_except(prog, except_global));
    }
}

pub fn apply(prog: &mut Program) {
    for (global, expr) in &prog.globals {
        expr.borrow_mut().inline_globals_except(prog, global);
    }
}
