use std::collections::HashMap;
use std::str::Chars;
use std::str::FromStr;

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
            self.next();
            if self.ch == '/' {
                self.next();
                loop {
                    if self.ch == '\n' {
                        self.next();
                        break;
                    } else if self.ch == '\0' {
                        return Ok(());
                    } else {
                        self.next();
                    }
                }
            } else if self.ch == '*' {
                self.next();
                loop {
                    if self.ch == '*' {
                        self.next();
                        if self.ch == '/' {
                            self.next();
                            break;
                        }
                    } else if self.ch == '\0' {
                        return Err(Error::UnexpectedEndOfJson);
                    } else {
                        self.next();
                    }
                }
            } else {
                return Err(Error::UnexpectedCharacter);
            }
            while self.ch.is_ascii_whitespace() {
                self.next();
            }
        }
        Ok(())
    }

    pub fn parse_value(&mut self) -> Result<Value, Error> {
        match self.ch {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_boolean(),
            '+' | '-' | '.' | '0'..='9' | 'I' | 'N' => self.parse_number(),
            '"' => self.parse_string(),
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
        let mut s = String::with_capacity(16);

        if self.ch == '+' || self.ch == '-' {
            s.push(self.ch);
            self.next();
        }

        if self.ch == '0' {
            self.next();
            if self.ch == '0' {
                return Err(Error::UnparseableNumber);
            } else if self.ch == 'x' || self.ch == 'X' {
                self.next();
                loop {
                    match self.ch {
                        '0'..='9' | 'a'..='f' | 'A'..='F' => (),
                        _ => break,
                    }
                    s.push(self.ch);
                    self.next();
                }
                return match i32::from_str_radix(&s, 16) {
                    Ok(i) => Ok(Value::Integer(i)),
                    Err(_) => Err(Error::UnparseableNumber),
                };
            } else {
                s.push('0');
            }
        } else if self.ch == 'I' {
            self.next();
            self.expect_sequence("nfinity")?;
            return if s == "-" {
                Ok(Value::Float(std::f64::NEG_INFINITY))
            } else {
                Ok(Value::Float(std::f64::INFINITY))
            };
        } else if self.ch == 'N' {
            self.next();
            self.expect_sequence("aN")?;
            return Ok(Value::Float(std::f64::NAN));
        }

        let mut is_float = false;
        loop {
            match self.ch {
                '.' | 'e' | 'E' => is_float = true,
                '+' | '-' | '0'..='9' => (),
                _ => break,
            }
            s.push(self.ch);
            self.next();
        }
        if is_float {
            match f64::from_str(&s) {
                Ok(f) => Ok(Value::Float(f)),
                Err(_) => Err(Error::UnparseableNumber),
            }
        } else {
            match i32::from_str(&s) {
                Ok(i) => Ok(Value::Integer(i)),
                Err(_) => Err(Error::UnparseableNumber),
            }
        }
    }

    fn parse_string(&mut self) -> Result<Value, Error> {
        let mut s = String::with_capacity(64);

        self.next();

        while self.ch != '"' {
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
