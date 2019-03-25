use std::slice;
use super::{
    lex::{Lexeme, Token},
    ast::{
        Program,
        Def,
        Expr,
        Literal,
        Builtin,
    },
};
use crate::{
    Error,
    Expected,
    Unexpected,
};

fn read_params(tokens: &mut slice::Iter<Token>) -> Result<Vec<(String, (usize, usize))>, Error> {
    match tokens.next() {
        Some(Token(Lexeme::Pipe, _)) => {},
        Some(Token(_, range)) => return Err(Error::expected(Expected::Pipe).at(*range)),
        None => return Err(Error::expected(Expected::Pipe)),
    }

    let mut params = Vec::new();

    // Read parameter identifiers until a pipe, or a non-identifier
    while let Some(token) = tokens.next() {
        match token {
            Token(Lexeme::Ident(ident, arity), _) => params.push((ident.clone(), *arity)),
            Token(Lexeme::Pipe, _) => return Ok(params),
            Token(_, range) => return Err(Error::expected(Expected::ArityIdent).at(*range)),
        }
    }

    // We ran out of tokens, yet didn't find a trailing pipe!
    Err(Error::expected_delimiter('|'))
}

const BUILTINS: [(&'static str, (usize, usize)); 14] = [
    ("__head", (1, 1)),
    ("__tail", (1, 1)),
    ("__wrap", (1, 1)),
    ("__cat", (2, 1)),

    ("__add", (2, 1)),
    ("__sub", (2, 1)),
    ("__mul", (2, 1)),
    ("__div", (2, 1)),
    ("__rem", (2, 1)),
    ("__eq", (2, 1)),
    ("__less", (2, 1)),
    ("__lesseq", (2, 1)),

    ("__input", (1, 2)),
    ("__print", (2, 1)),
];

fn is_builtin(name: &str) -> bool {
    (&BUILTINS).iter().find(|(b, _)| *b == name).is_some()
}

fn read_builtin(
    name: &str,
    tokens: &mut slice::Iter<Token>,
    globals: &Vec<(String, (usize, usize))>,
    locals: &Vec<(String, (usize, usize))>,
) -> Result<Builtin, Error> {
    Ok(match name {
        "__head" => Builtin::Head(read_args(tokens, 1, globals, locals)?.0),
        "__tail" => Builtin::Tail(read_args(tokens, 1, globals, locals)?.0),
        "__wrap" => Builtin::Wrap(read_args(tokens, 1, globals, locals)?.0),
        "__cat" => Builtin::Cat(read_args(tokens, 2, globals, locals)?.0),

        "__input" => Builtin::Input(read_args(tokens, 1, globals, locals)?.0),
        "__print" => Builtin::Input(read_args(tokens, 2, globals, locals)?.0),

        "__add" => Builtin::Add(read_args(tokens, 2, globals, locals)?.0),
        "__sub" => Builtin::Sub(read_args(tokens, 2, globals, locals)?.0),
        "__mul" => Builtin::Mul(read_args(tokens, 2, globals, locals)?.0),
        "__div" => Builtin::Div(read_args(tokens, 2, globals, locals)?.0),
        "__rem" => Builtin::Rem(read_args(tokens, 2, globals, locals)?.0),
        "__eq" => Builtin::Eq(read_args(tokens, 2, globals, locals)?.0),
        "__less" => Builtin::Less(read_args(tokens, 2, globals, locals)?.0),
        "__lesseq" => Builtin::LessEq(read_args(tokens, 2, globals, locals)?.0),
        _ => unimplemented!(),
    })
}

fn read_args(
    tokens: &mut slice::Iter<Token>,
    args: isize,
    globals: &Vec<(String, (usize, usize))>,
    locals: &Vec<(String, (usize, usize))>,
) -> Result<(Vec<Expr>, isize), Error> {
    let get_ident_arity = |ident: &str| (&BUILTINS)
        .iter()
        .map(|b| *b)
        .chain(locals.iter().map(|(l, a)| (l.as_str(), *a)).rev())
        .chain(globals.iter().map(|(g, a)| (g.as_str(), *a)).rev())
        .find(|(name, _)| *name == ident)
        .map(|(_, arity)| arity);

    let mut exprs = Vec::new();

    let mut args_left = args;
    while args_left > 0 {
        match tokens.clone().next().ok_or(Error::unexpected_eof())? {
            Token(Lexeme::Num(num), range) => {
                // Confirm reading num
                tokens.next();
                args_left -= 1;
                exprs.push(Expr::Literal(Literal::Num(
                    num.parse().map_err(|_| Error::bad_number().at(*range))?
                )))
            },
            Token(Lexeme::True, range) => {
                // Confirm reading true
                tokens.next();
                args_left -= 1;
                exprs.push(Expr::Literal(Literal::Bool(true)))
            },
            Token(Lexeme::False, range) => {
                // Confirm reading false
                tokens.next();
                args_left -= 1;
                exprs.push(Expr::Literal(Literal::Bool(false)))
            },
            Token(Lexeme::Null, range) => {
                // Confirm reading null
                tokens.next();
                args_left -= 1;
                exprs.push(Expr::Literal(Literal::Null))
            },
            Token(Lexeme::Ident(name, ident_arity), range) => {

                if let Some(arity) = get_ident_arity(name) {
                    args_left -= arity.1 as isize;

                    if args_left >= 0 {
                        tokens.next(); // Confirm reading ident
                    } else {
                        break;
                    }

                    if *ident_arity != (0, 1) {
                        return Err(Error::expected(Expected::NoArityIdent).at(*range));
                    } else if is_builtin(&name) {
                        exprs.push(Expr::Builtin(read_builtin(name, tokens, globals, locals)?));
                    } else {
                        exprs.push(Expr::Call(
                            name.clone(),
                            read_args(tokens, arity.0 as isize, globals, locals)?.0,
                        ));
                    }
                } else {
                    tokens.next(); // Confirm reading ident

                    return Err(Error::unknown_ident(name.clone()).at(*range));
                }
            },
            Token(Lexeme::Pipe, range) => {
                let params = read_params(tokens)?;
                if params.len() != 1 {
                    return Err(Error::one_param_only().at(*range));
                } else {
                    let param = params.into_iter().next().unwrap();

                    let mut body_locals = locals.clone();
                    body_locals.push(param.clone());

                    let (body, args_read) = read_args(tokens, args_left, globals, &body_locals)?;
                    exprs.push(Expr::Closure(
                        param,
                        body,
                    ));

                    args_left -= args_read;
                }
            },
            Token(Lexeme::If, range) => {
                tokens.next(); // Confirm reading 'if'

                let (args, args_read) = read_args(tokens, 1 + args_left * 2, globals, locals)?;

                exprs.push(Expr::If(args));

                args_left -= (args_read - 1) / 2;
            },
            Token(Lexeme::OpenParen, range) => {
                tokens.next(); // Confirm reading '('

                loop {
                    let (mut args, args_read) = read_args(tokens, args_left, globals, locals)?;

                    args_left -= args_read;

                    if args_left < 0 {
                        return Err(Error::incorrect_arity().at(*range));
                    } else {
                        exprs.append(&mut args);
                    }

                    match tokens.clone().next().ok_or(Error::unexpected_eof())? {
                        Token(Lexeme::CloseParen, range) => {
                            tokens.next(); // Confirm reading ')'
                            break;
                        },
                        Token(Lexeme::Def, range) => {
                            return Err(Error::unexpected(Unexpected::Def).at(*range));
                        },
                        _ => {},
                    }
                }
            },
            Token(Lexeme::CloseParen, range) => {
                break;
            },
            Token(Lexeme::Def, range) => {
                tokens.next(); // Confirm reading 'def'

                return Err(Error::unexpected(Unexpected::Def).at(*range));
            },
            t => unimplemented!("{:?}", t),
        }
    }

    Ok((exprs, args - args_left))
}

fn gen_global_arities(
    mut tokens: slice::Iter<Token>,
) -> Result<Vec<(String, (usize, usize))>, Error> {
    let mut arities = Vec::new();

    // Keep reading tokens
    while let Some(token) = tokens.next() {
        // When we find a 'def', read its name and argument list
        if let Token(Lexeme::Def, range) = token {
            // Get the name of the function
            let (name, arity) = match tokens.next().ok_or(Error::unexpected_eof())? {
                Token(Lexeme::Ident(name, arity), _) => (name, *arity),
                Token(_, range) =>
                    return Err(Error::expected(Expected::NoArityIdent).at(*range)),
            };

            // Add the global to the list
            arities.push((name.clone(), arity));
        }
    }

    Ok(arities)
}

pub fn parse_program(mut tokens: slice::Iter<Token>) -> Result<Program, Error> {
    let global_arities = gen_global_arities(tokens.clone())?;

    let mut prog = Program::new();

    while let Some(token) = tokens.next() {
        match token {
            Token(Lexeme::Def, range) => match tokens.next() {
                Some(Token(Lexeme::Ident(name, arity), range)) => {
                    let body = read_args(&mut tokens, arity.1 as isize, &global_arities, &Vec::new())?.0;
                    prog.globals.insert(
                        name.to_string(),
                        Def::new(*arity, body),
                    );
                },
                Some(Token(_, range)) => {
                    return Err(Error::expected(Expected::ArityIdent).at(*range));
                },
                _ => return Err(Error::unexpected_eof()),
            },
            Token(_, range) => {
                return Err(Error::expected(Expected::Def).at(*range))
            },
        }
    }

    Ok(prog)
}

#[cfg(test)]
mod tests {
    use super::{
        *,
        super::lex::lex,
    };

    #[test]
    fn arities() {
        let code = "
            def add'' |x| |y|
                + x y

            def foo' |x''.|
                x 10 20

            def five_and_six.
                (5 6)

            def no_args
                |x| + 5 x

            def bar''' |a'.| |b''.| |c'''.|
                c b a 1 2 3 4 5

            def main' ||
                10
        ";

        assert_eq!(
            gen_global_arities(lex(code).unwrap().iter()),
            Ok(vec![
                ("add".to_string(), (2, 1)),
                ("foo".to_string(), (1, 1)),
                ("five_and_six".to_string(), (0, 2)),
                ("no_args".to_string(), (0, 1)),
                ("bar".to_string(), (3, 1)),
                ("main".to_string(), (1, 1)),
            ])
        );
    }
}
