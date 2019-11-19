mod error;
mod parser;
mod value;

pub use crate::error::Error;
use crate::parser::Parser;
pub use crate::value::Value;

pub fn parse(json: String) -> Result<Value, Error> {
    let mut parser = Parser {
        chars: json.chars(),
        ch: '\0',
    };
    parser.next();
    parser.skip_comments()?;
    parser.parse_value()
}
