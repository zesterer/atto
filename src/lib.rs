pub mod parse;
pub mod ir;
pub mod exec;

use parse::src::SrcRange;

#[derive(Debug, PartialEq)]
pub struct Error {
    kind: ErrorKind,
    range: Option<SrcRange>,
}

impl Error {
    pub fn unexpected_char(c: char) -> Self {
        Self {
            kind: ErrorKind::UnexpectedChar(c),
            range: None,
        }
    }

    pub fn expected_delimiter(c: char) -> Self {
        Self {
            kind: ErrorKind::ExpectedDelimiter(c),
            range: None,
        }
    }


    pub fn expected(expected: Expected) -> Self {
        Self {
            kind: ErrorKind::Expected(expected),
            range: None,
        }
    }

    pub fn unexpected_eof() -> Self {
        Self {
            kind: ErrorKind::Unexpected(Unexpected::Eof),
            range: None,
        }
    }

    pub fn unexpected(unexpected: Unexpected) -> Self {
        Self {
            kind: ErrorKind::Unexpected(unexpected),
            range: None,
        }
    }

    pub fn bad_number() -> Self {
        Self {
            kind: ErrorKind::BadNumber,
            range: None,
        }
    }

    pub fn unknown_ident(name: String) -> Self {
        Self {
            kind: ErrorKind::UnknownIdent(name),
            range: None,
        }
    }

    pub fn incorrect_arity() -> Self {
        Self {
            kind: ErrorKind::IncorrectArity,
            range: None,
        }
    }

    pub fn one_param_only() -> Self {
        Self {
            kind: ErrorKind::OneParamOnly,
            range: None,
        }
    }

    pub fn at(mut self, range: SrcRange) -> Self {
        self.range = Some(range);
        self
    }
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    UnexpectedChar(char), // An unexpected character was encountered
    ExpectedDelimiter(char), // A delimiter was expected but was never found
    Expected(Expected), // Expected the given thing, but found something else instead
    Unexpected(Unexpected), // An unexpected thing was found
    BadNumber, // The numerical format was not recognised
    UnknownIdent(String), // The identifier was not found in the current scope
    IncorrectArity, // The parser tried to parse an expression but found an unbalanced net arity
    OneParamOnly, // Closures are only permitted to have a single parameter
}

#[derive(Debug, PartialEq)]
pub enum Expected {
    ArityIdent,
    NoArityIdent,
    Pipe,
    Def,
    CloseParen,
}

#[derive(Debug, PartialEq)]
pub enum Unexpected {
    Eof,
    Def,
}
