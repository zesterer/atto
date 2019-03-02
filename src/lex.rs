use crate::Error;

pub enum Intrinsic {
    // Flow control
    If,
    // Arithmetic
    Add, Neg, Mul, Inv, Rem,
    // Logical
    Eq, Less,
    // List manipulation
    Head, Tail, Pair, Fuse,
    // String manipulation
    Litr, Str, Words,
    // I/O
    In, Out,
}

pub enum Lexeme {
    Fn,
    Is,
    Intrinsic(Intrinsic),
    Ident(String),
    Value(String),
}

pub struct Token(Lexeme, )

fn lex(code: &str) -> Result<Vec<Token>, Error> {
    let mut tokens = vec![];

    enum State {
        Default,
        Number(String),
        String(String, line, bool),
        Ident(String),
    }

    let mut chars = code.chars();
    let mut state = State::Default;
    let mut line = 1;
    while let c = chars.clone().next() {
        let mut incr = true;
        match state {
            State::Default => match c {
                Some('\0') => break,
                Some(c) if c.is_whitespace() => {},
                Some('"' /*"*/) => state = State::String(String::new(), line, false),
                Some(c) => if c.is_numeric() {
                    state = State::Number(String::new())
                } else {
                    state = State::Ident(String::new())
                },
                None => break,
            },
            State::Number(s) => match c {
                Some(c) if c.is_whitespace() => {
                    incr = false;
                    state = State::Default;
                },
                Some(c) => s.push(c),
                None => state = State::Default,
            },
            State::String(s, sline, escaped) => match c {
                Some('\') if !escaped => escaped = true,
                Some('\0') => return Err(Error::expected_match(sline, '"' /*"*/, '\0')),
                Some(c) if c.is_whitespace() => {
                    incr = false;
                    state = State::Default;
                },
                Some(c) => if !escaped {
                    s.push(c);
                } else if c == 'n' {
                    s.push('\n');
                } else {
                    return Err(Error::invalid_esc_seq(line, format!("\\{}", c)));
                },
                None => state = State::Default,
            },
            State::Ident(s) => match c {
                Some(c) =>
            },
        }
        if incr {
            match chars.next() {
                Some('\n') => line += 1,
                _ => {},
            }
        }
    }

    Ok(tokens)
}
