use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;
use std::str::FromStr;

use crate::error::Error;
use crate::value::Value;

pub struct Parser<'a> {
    pub chars: Peekable<Chars<'a>>,
    pub ch: char,
}

impl<'a> Parser<'a> {
    pub fn next(&mut self) {
        self.ch = match self.chars.next() {
            Some(ch) => ch,
            None => '\0',
        };
    }

    fn peek(&mut self) -> char {
        match self.chars.peek() {
            Some(ch) => *ch,
            None => '\0',
        }
    }

    fn expect(&mut self, ch: char) -> Result<(), Error> {
        if self.ch == '\0' {
            return Err(Error::UnexpectedEndOfJson);
        } else if self.ch != ch {
            return Err(Error::UnexpectedCharacter);
        }
        self.next();
        Ok(())
    }

    fn expect_sequence(&mut self, s: &str) -> Result<(), Error> {
        for ch in s.chars() {
            self.expect(ch)?;
        }
        Ok(())
    }

    pub fn skip_comments(&mut self) -> Result<(), Error> {
        while self.ch.is_ascii_whitespace() {
            self.next();
        }
        while self.ch == '/' {
            match self.peek() {
                '/' => self.skip_single_line_comment(),
                '*' => self.skip_multi_line_comment()?,
                _ => return Err(Error::UnexpectedCharacter),
            }
            while self.ch.is_ascii_whitespace() {
                self.next();
            }
        }
        Ok(())
    }

    fn skip_single_line_comment(&mut self) {
        self.next();
        self.next();
        loop {
            if self.ch == '\n' {
                self.next();
                return;
            } else if self.ch == '\0' {
                return;
            }
            self.next();
        }
    }

    fn skip_multi_line_comment(&mut self) -> Result<(), Error> {
        self.next();
        self.next();
        loop {
            if self.ch == '*' && self.peek() == '/' {
                self.next();
                self.next();
                return Ok(());
            } else if self.ch == '\0' {
                return Err(Error::UnexpectedEndOfJson);
            }
            self.next();
        }
    }

    pub fn parse_value(&mut self) -> Result<Value, Error> {
        match self.ch {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_boolean(),
            '0'..='9' | '+' | '-' | '.' | 'I' | 'N' => self.parse_number(),
            '"' | '\'' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            _ => Err(Error::UnexpectedCharacter),
        }
    }

    fn parse_null(&mut self) -> Result<Value, Error> {
        self.next();
        self.expect_sequence("ull")?;
        Ok(Value::Null)
    }

    fn parse_boolean(&mut self) -> Result<Value, Error> {
        if self.ch == 't' {
            self.next();
            self.expect_sequence("rue")?;
            Ok(Value::Boolean(true))
        } else {
            self.next();
            self.expect_sequence("alse")?;
            Ok(Value::Boolean(false))
        }
    }

    fn parse_number(&mut self) -> Result<Value, Error> {
        let sign = match self.ch {
            '+' | '-' => Some(self.ch),
            _ => None,
        };

        if sign.is_some() {
            self.next();
        }

        match self.ch {
            '0' => match self.peek() {
                '0'..='9' => Err(Error::UnparseableNumber),
                'x' | 'X' => self.parse_hex_number(sign),
                _ => self.parse_decimal_number(sign),
            },
            'I' => self.parse_infinity(sign),
            'N' => self.parse_nan(),
            _ => self.parse_decimal_number(sign),
        }
    }

    fn parse_hex_number(&mut self, sign: Option<char>) -> Result<Value, Error> {
        let mut buf = String::with_capacity(16);

        if let Some(ch) = sign {
            buf.push(ch);
        }
        self.next();
        self.next();

        loop {
            match self.ch {
                '0'..='9' | 'a'..='f' | 'A'..='F' => (),
                _ => break,
            }
            buf.push(self.ch);
            self.next();
        }

        return match i32::from_str_radix(&buf, 16) {
            Ok(i) => Ok(Value::Integer(i)),
            Err(_) => Err(Error::UnparseableNumber),
        };
    }

    fn parse_decimal_number(&mut self, sign: Option<char>) -> Result<Value, Error> {
        let mut is_float = false;
        let mut buf = String::with_capacity(16);

        if let Some(ch) = sign {
            buf.push(ch);
        }

        loop {
            match self.ch {
                '.' | 'e' | 'E' => is_float = true,
                '0'..='9' | '+' | '-' => (),
                _ => break,
            }
            buf.push(self.ch);
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
        self.expect_sequence("nfinity")?;
        match sign {
            Some('-') => Ok(Value::Float(std::f64::NEG_INFINITY)),
            _ => Ok(Value::Float(std::f64::INFINITY)),
        }
    }

    fn parse_nan(&mut self) -> Result<Value, Error> {
        self.next();
        self.expect_sequence("aN")?;
        Ok(Value::Float(std::f64::NAN))
    }

    fn parse_string(&mut self) -> Result<Value, Error> {
        let mark = self.ch; // " or '
        let mut s = String::with_capacity(64);

        self.next();

        while self.ch != mark {
            if self.ch == '\0' {
                return Err(Error::UnexpectedEndOfJson);
            }
            s.push(self.ch);
            self.next();
        }
        self.next();

        Ok(Value::String(s))
    }

    fn parse_array(&mut self) -> Result<Value, Error> {
        let mut v = Vec::new();

        self.next();
        self.skip_comments()?;

        while self.ch != ']' {
            let value = self.parse_value()?;
            v.push(value);

            self.skip_comments()?;
            match self.ch {
                ',' => self.next(),
                ']' => break,
                '\0' => return Err(Error::UnexpectedEndOfJson),
                _ => return Err(Error::UnexpectedCharacter),
            }
            self.skip_comments()?;
        }
        self.next();

        Ok(Value::Array(v))
    }

    fn parse_object(&mut self) -> Result<Value, Error> {
        let mut m = HashMap::new();

        self.next();
        self.skip_comments()?;

        while self.ch != '}' {
            let key = match self.parse_value() {
                Ok(Value::String(s)) => s,
                Err(e) => return Err(e),
                _ => return Err(Error::UnexpectedCharacter),
            };

            self.skip_comments()?;
            self.expect(':')?;
            self.skip_comments()?;

            let value = self.parse_value()?;
            m.insert(key, value);

            self.skip_comments()?;
            match self.ch {
                ',' => self.next(),
                '}' => break,
                '\0' => return Err(Error::UnexpectedEndOfJson),
                _ => return Err(Error::UnexpectedCharacter),
            }
            self.skip_comments()?;
        }
        self.next();

        Ok(Value::Object(m))
    }
}
