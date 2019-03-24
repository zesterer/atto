use std::slice;
use super::{
    lex::{Lexeme, Token},
    ast::{
        Prog,
        Func,
    },
};
use crate::{
    Error,
    Expected,
};

fn read_params(mut tokens: slice::Iter<Token>) -> Result<Vec<(String, usize)>, Error> {
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

fn gen_global_arities(
    mut tokens: slice::Iter<Token>,
) -> Result<Vec<(String, usize)>, Error> {
    let mut arities = Vec::new();

    // Keep reading tokens
    while let Some(token) = tokens.next() {
        // When we find a 'def', read its name and argument list
        if let Token(Lexeme::Def, range) = token {
            // Get the name of the function
            let name = match tokens.next().ok_or(Error::expected_more(*range))? {
                Token(Lexeme::Ident(name, 0), _) => name,
                Token(_, range) =>
                    return Err(Error::expected(Expected::NoArityIdent).at(*range)),
            };

            // Get the parameter list of the function
            let params = read_params(tokens.clone())?;

            // Add the global to the list
            arities.push((name.clone(), params.len()));
        }
    }

    Ok(arities)
}

pub fn parse_prog(mut tokens: slice::Iter<Token>) -> Result<Prog, Error> {
    let global_arities = gen_global_arities(tokens.clone());

    unimplemented!();
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
            def add |x y|
                + x y

            def foo |x''|
                x 10 20

            def no_args ||
                |x| + 5 x

            def bar |a' b'' c'''|
                c b a 1 2 3 4 5

            def main ||
                10
        ";

        assert_eq!(
            gen_global_arities(lex(code).unwrap().iter()),
            Ok(vec![
                ("add".to_string(), 2),
                ("foo".to_string(), 1),
                ("no_args".to_string(), 0),
                ("bar".to_string(), 3),
                ("main".to_string(), 0),
            ])
        );
    }
}
