pub mod src;
pub mod lex;
pub mod parse;
pub mod ast;

use self::{
    ast::Program,
    lex::lex,
    parse::parse_program,
};
use crate::Error;

pub fn code(code: &str) -> Result<Program, Vec<Error>> {
    parse_program(lex(code)?.iter()).map_err(|err| vec![err])
}
