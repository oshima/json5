use std::collections::HashMap;
use std::str::Chars;

use crate::error::Error;
use crate::value::Value;

pub struct Parser<'a> {
    pub chars: Chars<'a>,
    pub ch: char,
}

impl<'a> Parser<'a> {
    pub fn next(&mut self) {
        self.ch = match self.chars.next() {
            Some(ch) => ch,
            None => '\0',
        };
    }

    fn expect(&mut self, ch: char) -> Result<(), Error> {
        if self.ch != ch {
            return Err(Error::UnexpectedCharacter);
        }
        self.next();
        Ok(())
    }

    pub fn skip_comments(&mut self) -> Result<(), Error> {
        while self.ch.is_ascii_whitespace() {
            self.next();
        }
        while self.ch == '/' {
            self.next();
            match self.ch {
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
        loop {
            match self.ch {
                '\n' => break,
                '\0' => return,
                _ => self.next(),
            }
        }
        self.next();
    }

    fn skip_multi_line_comment(&mut self) -> Result<(), Error> {
        self.next();
        loop {
            match self.ch {
                '*' => {
                    self.next();
                    if self.ch == '/' {
                        break;
                    }
                },
                '\0' => return Err(Error::UnexpectedEndOfJson),
                _ => self.next(),
            }
        }
        self.next();
        Ok(())
    }

    pub fn parse_value(&mut self) -> Result<Value, Error> {
        match self.ch {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_boolean(),
            '+' | '-' | '0'..='9' => self.parse_number(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            _ => Err(Error::UnexpectedCharacter),
        }
    }

    fn parse_null(&mut self) -> Result<Value, Error> {
        self.next();
        self.expect('u')?;
        self.expect('l')?;
        self.expect('l')?;
        Ok(Value::Null)
    }

    fn parse_boolean(&mut self) -> Result<Value, Error> {
        if self.ch == 't' {
            self.next();
            self.expect('r')?;
            self.expect('u')?;
            self.expect('e')?;
            Ok(Value::Boolean(true))
        } else {
            self.next();
            self.expect('a')?;
            self.expect('l')?;
            self.expect('s')?;
            self.expect('e')?;
            Ok(Value::Boolean(false))
        }
    }

    fn parse_number(&mut self) -> Result<Value, Error> {
        let mut n = 0;
        let sign = if self.ch == '-' { -1 } else { 1 };

        if self.ch == '+' || self.ch == '-' {
            self.next();
        }
        while self.ch.is_ascii_digit() {
            n *= 10;
            n += (self.ch as i32) - ('0' as i32);
            self.next();
        }
        Ok(Value::Number(sign * n))
    }

    fn parse_string(&mut self) -> Result<Value, Error> {
        let mut s = String::new();

        self.next();

        while self.ch != '"' {
            match self.ch {
                '"' => break,
                '\0' => return Err(Error::UnexpectedEndOfJson),
                _ => s.push(self.ch),
            }
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
