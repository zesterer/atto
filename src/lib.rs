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

    pub fn expected_more(range: SrcRange) -> Self {
        Self {
            kind: ErrorKind::ExpectedMore(range),
            range: None,
        }
    }

    pub fn expected(expected: Expected) -> Self {
        Self {
            kind: ErrorKind::Expected(expected),
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
    ExpectedMore(SrcRange), // One or more tokens were expected but were never found
    Expected(Expected), // Expected the given thing, but found something else instead
}

#[derive(Debug, PartialEq)]
pub enum Expected {
    ArityIdent,
    NoArityIdent,
    Pipe,
}
