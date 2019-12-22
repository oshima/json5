use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;
use std::str::FromStr;

use crate::error::Error;
use crate::value::Value;

pub struct Parser<'a> {
    pub chars: Peekable<Chars<'a>>,
    pub ch: Option<char>,
}

impl<'a> Parser<'a> {
    pub fn next(&mut self) {
        self.ch = self.chars.next();
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn expect(&mut self, ch: char) -> Result<(), Error> {
        match self.ch {
            None => Err(Error::UnexpectedEndOfJson),
            Some(c) if c == ch => Ok(()),
            _ => Err(Error::UnexpectedCharacter),
        }
    }

    fn consume(&mut self, ch: char) -> Result<(), Error> {
        self.expect(ch)?;
        self.next();
        Ok(())
    }

    fn consume_sequence(&mut self, s: &str) -> Result<(), Error> {
        for ch in s.chars() {
            self.expect(ch)?;
            self.next();
        }
        Ok(())
    }

    pub fn skip_comments(&mut self) -> Result<(), Error> {
        self.skip_whitespace();

        while let Some('/') = self.ch {
            match self.peek() {
                Some('/') => self.skip_single_line_comment(),
                Some('*') => self.skip_multi_line_comment()?,
                None => return Err(Error::UnexpectedEndOfJson),
                _ => return Err(Error::UnexpectedCharacter),
            }
            self.skip_whitespace();
        }
        Ok(())
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.ch {
            if !c.is_ascii_whitespace() {
                return;
            }
            self.next();
        }
    }

    fn skip_single_line_comment(&mut self) {
        self.next();
        self.next();

        while let Some(c) = self.ch {
            self.next();
            if c == '\n' {
                return;
            }
        }
    }

    fn skip_multi_line_comment(&mut self) -> Result<(), Error> {
        self.next();
        self.next();

        while let Some(c) = self.ch {
            self.next();
            if c != '*' {
                continue;
            }
            if let Some('/') = self.ch {
                self.next();
                return Ok(());
            }
        }
        Err(Error::UnexpectedEndOfJson)
    }

    pub fn parse_value(&mut self) -> Result<Value, Error> {
        match self.ch {
            None => Err(Error::UnexpectedEndOfJson),
            Some(c) => match c {
                'n' => self.parse_null(),
                't' | 'f' => self.parse_boolean(),
                '0'..='9' | '+' | '-' | '.' | 'I' | 'N' => self.parse_number(),
                '"' | '\'' => self.parse_string(),
                '[' => self.parse_array(),
                '{' => self.parse_object(),
                _ => Err(Error::UnexpectedCharacter),
            },
        }
    }

    fn parse_null(&mut self) -> Result<Value, Error> {
        self.next();
        self.consume_sequence("ull")?;
        Ok(Value::Null)
    }

    fn parse_boolean(&mut self) -> Result<Value, Error> {
        if self.ch.unwrap() == 't' {
            self.next();
            self.consume_sequence("rue")?;
            Ok(Value::Boolean(true))
        } else {
            self.next();
            self.consume_sequence("alse")?;
            Ok(Value::Boolean(false))
        }
    }

    fn parse_number(&mut self) -> Result<Value, Error> {
        let sign = match self.ch.unwrap() {
            '+' | '-' => self.ch,
            _ => None,
        };

        if sign.is_some() {
            self.next();
        }

        match self.ch {
            None => Err(Error::UnexpectedEndOfJson),
            Some(c) => match c {
                '0' => match self.peek() {
                    None => self.parse_decimal_number(sign),
                    Some(c) => match c {
                        '0'..='9' => Err(Error::UnparseableNumber),
                        'x' | 'X' => self.parse_hex_number(sign),
                        _ => self.parse_decimal_number(sign),
                    },
                },
                'I' => self.parse_infinity(sign),
                'N' => self.parse_nan(),
                _ => self.parse_decimal_number(sign),
            },
        }
    }

    fn parse_hex_number(&mut self, sign: Option<char>) -> Result<Value, Error> {
        let mut buf = String::with_capacity(16);

        if let Some(c) = sign {
            buf.push(c);
        }
        self.next();
        self.next();

        while let Some(c) = self.ch {
            if !c.is_ascii_hexdigit() {
                break;
            }
            buf.push(c);
            self.next();
        }

        match i32::from_str_radix(&buf, 16) {
            Ok(i) => Ok(Value::Integer(i)),
            Err(_) => Err(Error::UnparseableNumber),
        }
    }

    fn parse_decimal_number(&mut self, sign: Option<char>) -> Result<Value, Error> {
        let mut is_float = false;
        let mut buf = String::with_capacity(16);

        if let Some(c) = sign {
            buf.push(c);
        }

        while let Some(c) = self.ch {
            match c {
                '0'..='9' | '+' | '-' => (),
                '.' | 'e' | 'E' => is_float = true,
                _ => break,
            }
            buf.push(c);
            self.next();
        }

        if is_float {
            match f64::from_str(&buf) {
                Ok(f) => Ok(Value::Float(f)),
                Err(_) => Err(Error::UnparseableNumber),
            }
        } else {
            match i32::from_str(&buf) {
                Ok(i) => Ok(Value::Integer(i)),
                Err(_) => Err(Error::UnparseableNumber),
            }
        }
    }

    fn parse_infinity(&mut self, sign: Option<char>) -> Result<Value, Error> {
        self.next();
        self.consume_sequence("nfinity")?;
        match sign {
            Some('-') => Ok(Value::Float(std::f64::NEG_INFINITY)),
            _ => Ok(Value::Float(std::f64::INFINITY)),
        }
    }

    fn parse_nan(&mut self) -> Result<Value, Error> {
        self.next();
        self.consume_sequence("aN")?;
        Ok(Value::Float(std::f64::NAN))
    }

    fn parse_string(&mut self) -> Result<Value, Error> {
        let mark = self.ch.unwrap(); // " or '
        let mut s = String::with_capacity(64);

        self.next();

        while let Some(c) = self.ch {
            self.next();
            if c == mark {
                return Ok(Value::String(s));
            }
            s.push(c);
        }
        Err(Error::UnexpectedEndOfJson)
    }

    fn parse_array(&mut self) -> Result<Value, Error> {
        let mut v = Vec::new();

        self.next();
        self.skip_comments()?;

        while let Some(c) = self.ch {
            if c == ']' {
                self.next();
                return Ok(Value::Array(v));
            }

            v.push(self.parse_value()?);
            self.skip_comments()?;

            match self.ch {
                None => break,
                Some(']') => {
                    self.next();
                    return Ok(Value::Array(v));
                }
                Some(',') => {
                    self.next();
                    self.skip_comments()?;
                }
                _ => return Err(Error::UnexpectedCharacter),
            }
        }
        Err(Error::UnexpectedEndOfJson)
    }

    fn parse_object(&mut self) -> Result<Value, Error> {
        let mut m = HashMap::new();

        self.next();
        self.skip_comments()?;

        while let Some(c) = self.ch {
            if c == '}' {
                self.next();
                return Ok(Value::Object(m));
            }

            let key = match self.parse_value() {
                Ok(Value::String(s)) => s,
                Err(e) => return Err(e),
                _ => return Err(Error::UnexpectedCharacter),
            };

            self.skip_comments()?;
            self.consume(':')?;
            self.skip_comments()?;

            m.insert(key, self.parse_value()?);
            self.skip_comments()?;

            match self.ch {
                None => break,
                Some('}') => {
                    self.next();
                    return Ok(Value::Object(m));
                }
                Some(',') => {
                    self.next();
                    self.skip_comments()?;
                }
                _ => return Err(Error::UnexpectedCharacter),
            }
        }
        Err(Error::UnexpectedEndOfJson)
    }
}
