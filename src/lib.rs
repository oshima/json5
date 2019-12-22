mod error;
mod parser;
mod value;

pub use crate::error::Error;
use crate::parser::Parser;
pub use crate::value::Value;

pub fn parse(json: &str) -> Result<Value, Error> {
    let mut parser = Parser {
        chars: json.chars().peekable(),
        ch: None,
    };

    parser.next();
    parser.skip_comments()?;

    let value = parser.parse_value()?;

    parser.skip_comments()?;

    if parser.ch.is_some() {
        return Err(Error::UnexpectedCharacter);
    }
    Ok(value)
}
