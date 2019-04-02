use std::slice;
use super::{
    lex::{Lexeme, Token},
    ast::{
        Program,
        Expr,
        Decl,
        Literal,
        Builtin,
    },
};
use crate::{
    Error,
    Expected,
    Unexpected,
};

fn read_params(tokens: &mut slice::Iter<Token>) -> Result<Vec<(String, usize)>, Error> {
    match tokens.next() {
        Some(Token(Lexeme::Pipe, _)) => {},
        Some(Token(_, range)) => return Err(Error::expected(Expected::Pipe).at(*range)),
        None => return Err(Error::expected(Expected::Pipe)),
    }

    let mut params = Vec::new();

    // Read parameter identifiers until a pipe, or a non-identifier
    while let Some(token) = tokens.next() {
        match token {
            Token(Lexeme::Ident(ident, scalar, arity), range) => if *scalar {
                return Err(Error::expected(Expected::ArityIdent).at(*range));
            } else {
                params.push((ident.clone(), *arity))
            },
            Token(Lexeme::Pipe, _) => return Ok(params),
            Token(_, range) => return Err(Error::expected(Expected::Pipe).at(*range)),
        }
    }

    // We ran out of tokens, yet didn't find a trailing pipe!
    Err(Error::expected_delimiter('|'))
}

#[derive(Clone)]
enum ParseDecl {
    Single((String, usize)),
    Destructure(Vec<(String, usize)>),
}

fn read_decl(tokens: &mut slice::Iter<Token>) -> Result<ParseDecl, Error> {
    Ok(match tokens.clone().next() {
        Some(Token(Lexeme::Pipe, _)) => ParseDecl::Destructure(read_params(tokens)?),
        Some(Token(Lexeme::Ident(name, ident_scalar, ident_arity), _)) if !*ident_scalar => {
            tokens.next();
            ParseDecl::Single((name.clone(), *ident_arity))
        },
        Some(Token(_, range)) => return Err(Error::expected(Expected::ArityIdent).at(*range)),
        _ => return Err(Error::expected(Expected::ArityIdent)),
    })
}

const BUILTINS: [(&'static str, usize); 17] = [
    ("__head", 1),
    ("__tail", 1),
    ("__wrap", 1),
    ("__cat", 2),

    ("__add", 2),
    ("__sub", 2),
    ("__mul", 2),
    ("__div", 2),
    ("__rem", 2),
    ("__eq", 2),
    ("__less", 2),
    ("__lesseq", 2),
    ("__floor", 1),
    ("__ceil", 1),

    ("__input", 1),
    ("__print", 2),
    ("__debug", 1),
];

fn is_builtin(name: &str) -> bool {
    (&BUILTINS).iter().find(|(b, _)| *b == name).is_some()
}

fn read_builtin(
    name: &str,
    tokens: &mut slice::Iter<Token>,
    globals: &Vec<(String, usize)>,
    locals: &Vec<(String, usize)>,
) -> Result<Builtin, Error> {
    Ok(match name {
        "__head" => Builtin::Head(read_expr(tokens, globals, locals)?),
        "__tail" => Builtin::Tail(read_expr(tokens, globals, locals)?),
        "__wrap" => Builtin::Wrap(read_expr(tokens, globals, locals)?),
        "__cat" => Builtin::Cat(read_expr(tokens, globals, locals)?, read_expr(tokens, globals, locals)?),

        "__input" => Builtin::Input(read_expr(tokens, globals, locals)?),
        "__print" => Builtin::Print(read_expr(tokens, globals, locals)?, read_expr(tokens, globals, locals)?),
        "__debug" => Builtin::Debug(read_expr(tokens, globals, locals)?),

        "__add" => Builtin::Add(read_expr(tokens, globals, locals)?, read_expr(tokens, globals, locals)?),
        "__sub" => Builtin::Sub(read_expr(tokens, globals, locals)?, read_expr(tokens, globals, locals)?),
        "__mul" => Builtin::Mul(read_expr(tokens, globals, locals)?, read_expr(tokens, globals, locals)?),
        "__div" => Builtin::Div(read_expr(tokens, globals, locals)?, read_expr(tokens, globals, locals)?),
        "__rem" => Builtin::Rem(read_expr(tokens, globals, locals)?, read_expr(tokens, globals, locals)?),
        "__eq" => Builtin::Eq(read_expr(tokens, globals, locals)?, read_expr(tokens, globals, locals)?),
        "__less" => Builtin::Less(read_expr(tokens, globals, locals)?, read_expr(tokens, globals, locals)?),
        "__lesseq" => Builtin::LessEq(read_expr(tokens, globals, locals)?, read_expr(tokens, globals, locals)?),
        "__floor" => Builtin::Floor(read_expr(tokens, globals, locals)?),
        "__ceil" => Builtin::Ceil(read_expr(tokens, globals, locals)?),
        _ => unimplemented!(),
    })
}

fn read_expr(
    tokens: &mut slice::Iter<Token>,
    globals: &Vec<(String, usize)>,
    locals: &Vec<(String, usize)>,
) -> Result<Expr, Error> {
    let get_ident_arity = |ident: &str| (&BUILTINS)
        .iter()
        .map(|b| *b)
        .chain(globals.iter().map(|(g, a)| (g.as_str(), *a)))
        .chain(locals.iter().map(|(l, a)| (l.as_str(), *a)))
        .rev()
        .find(|(name, _)| *name == ident)
        .map(|(_, arity)| arity);

    Ok(match tokens.clone().next().ok_or(Error::unexpected_eof())? {
        Token(Lexeme::Num(num), range) => {
            // Confirm reading num
            tokens.next();
            Expr::Literal(Literal::Num(
                num.parse().map_err(|_| Error::bad_number().at(*range))?
            ))
        },
        Token(Lexeme::True, range) => {
            // Confirm reading true
            tokens.next();
            Expr::Literal(Literal::Bool(true))
        },
        Token(Lexeme::False, range) => {
            // Confirm reading false
            tokens.next();
            Expr::Literal(Literal::Bool(false))
        },
        Token(Lexeme::Null, range) => {
            // Confirm reading null
            tokens.next();
            Expr::Literal(Literal::Null)
        },
        Token(Lexeme::Str(s), range) => {
            // Confirm reading str
            tokens.next();
            Expr::Literal(Literal::Str(s.clone()))
        },
        Token(Lexeme::Ident(name, ident_scalar, ident_arity), range) => {
            tokens.next(); // Confirm reading ident

            if let Some(Token(Lexeme::Arrow, _)) = tokens.clone().next() {
                tokens.next(); // Confirm reading ':'

                let mut body_locals = locals.clone();
                body_locals.push((name.clone(), *ident_arity));

                let body = read_expr(tokens, globals, &body_locals)?;
                Expr::Closure(
                    Decl::Single(name.clone()),
                    Box::new(body),
                )
            } else if let Some(arity) = get_ident_arity(name) {
                if *ident_arity != 0 {
                    return Err(Error::expected(Expected::NoArityIdent).at(*range));
                } else if *ident_scalar {
                    Expr::Call(name.clone(), vec![])
                } else if is_builtin(&name) {
                    Expr::Builtin(Box::new(read_builtin(name, tokens, globals, locals)?))
                } else {
                    let mut args = Vec::new();
                    for _ in 0..arity {
                        args.push(read_expr(tokens, globals, locals)?);
                    }

                    Expr::Call(
                        name.clone(),
                        args,
                    )
                }
            } else {
                return Err(Error::unknown_ident(name.clone()).at(*range));
            }
        },
        Token(Lexeme::Pipe, range) => {
            let params = read_params(tokens)?;

            match tokens.clone().next() {
                Some(Token(Lexeme::Arrow, _)) => { tokens.next(); },
                Some(Token(_, range)) => return Err(Error::expected(Expected::Arrow).at(*range)),
                None => return Err(Error::expected(Expected::Arrow)),
            }

            let mut body_locals = locals.clone();
            body_locals.append(&mut params.clone());

            let body = read_expr(tokens, globals, &body_locals)?;
            Expr::Closure(
                Decl::Destructure(params.into_iter().map(|(ident, _)| ident).collect()),
                Box::new(body),
            )
        },
        Token(Lexeme::If, range) => {
            tokens.next(); // Confirm reading 'if'

            Expr::If(
                Box::new(read_expr(tokens, globals, locals)?),
                Box::new(read_expr(tokens, globals, locals)?),
                Box::new(read_expr(tokens, globals, locals)?),
            )
        },
        Token(Lexeme::Let, range) => {
            tokens.next(); // Confirm reading 'let'

            let decl = read_decl(tokens)?;

            let expr = read_expr(tokens, globals, locals)?;

            let mut then_locals = locals.clone();

            match decl.clone() {
                ParseDecl::Single(ident) => then_locals.push(ident),
                ParseDecl::Destructure(mut idents) => then_locals.append(&mut idents),
            }

            Expr::Let(
                match decl {
                    ParseDecl::Single((ident, _)) => Decl::Single(ident),
                    ParseDecl::Destructure(idents) =>
                        Decl::Destructure(idents.iter().cloned().map(|(ident, _)| ident).collect()),
                },
                Box::new(expr),
                Box::new(read_expr(tokens, globals, &then_locals)?),
            )
        },
        Token(Lexeme::Def, range) => {
            tokens.next(); // Confirm reading 'def'

            return Err(Error::unexpected(Unexpected::Def).at(*range));
        },
        t => unimplemented!("{:?}", t),
    })
}

fn gen_global_arities(
    mut tokens: slice::Iter<Token>,
) -> Result<Vec<(String, usize)>, Error> {
    let mut arities = Vec::new();

    // Keep reading tokens
    while let Some(token) = tokens.next() {
        // When we find a 'def', read its name and argument list
        if let Token(Lexeme::Def, range) = token {
            // Get the name of the function
            let (name, arity) = match tokens.next().ok_or(Error::unexpected_eof())? {
                Token(Lexeme::Ident(name, _, arity), _) => (name, *arity),
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
                Some(Token(Lexeme::Ident(name, scalar, arity), range)) => {
                    if *scalar {
                        return Err(Error::expected(Expected::ArityIdent).at(*range));
                    } else {
                        let body = read_expr(&mut tokens, &global_arities, &Vec::new())?;
                        prog.globals.insert(
                            name.to_string(),
                            body,
                        );
                    }
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

            def foo' |x''|
                x 10 20

            def no_args
                |x| + 5 x

            def bar''' |a'| |b''| |c'''|
                c b a 1 2 3 4 5

            def main' |@|
                10
        ";

        assert_eq!(
            gen_global_arities(lex(code).unwrap().iter()),
            Ok(vec![
                ("add".to_string(), 2),
                ("foo".to_string(), 1),
                ("no_args".to_string(), 0),
                ("bar".to_string(), 3),
                ("main".to_string(), 1),
            ])
        );
    }
}
