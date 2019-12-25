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

    fn peek(&mut self) -> Option<char> {
        match self.chars.peek() {
            None => None,
            Some(c) => Some(c.clone()),
        }
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
            match c {
                '\u{000A}' | '\u{000D}' => return Err(Error::UnexpectedCharacter),
                '\\' => match self.peek() {
                    None => break,
                    Some(c) => match c {
                        'x' => s.push_str(&self.parse_basic_latin_escape()?),
                        'u' => s.push_str(&self.parse_basic_multilingual_plane_escape()?),
                        '\'' | '"' | '\\' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' | '0' => {
                            s.push(self.parse_popular_character_escape(c));
                        }
                        '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}' => {
                            self.skip_line_continuation(c);
                        }
                        _ => {
                            self.next();
                            self.next();
                            s.push(c);
                        }
                    },
                },
                _ => {
                    self.next();
                    if c == mark {
                        return Ok(Value::String(s));
                    } else {
                        s.push(c);
                    }
                }
            }
        }
        Err(Error::UnexpectedEndOfJson)
    }

    fn parse_hex_utf8(&mut self) -> Result<u8, Error> {
        let mut num: u32 = 0;

        for _ in 0..2 {
            match self.ch {
                None => return Err(Error::UnexpectedEndOfJson),
                Some(c) => match c.to_digit(16) {
                    None => return Err(Error::UnexpectedCharacter),
                    Some(n) => num = num * 16 + n,
                },
            }
            self.next();
        }
        Ok(num as u8)
    }

    fn parse_hex_utf16(&mut self) -> Result<u16, Error> {
        let mut num: u32 = 0;

        for _ in 0..4 {
            match self.ch {
                None => return Err(Error::UnexpectedEndOfJson),
                Some(c) => match c.to_digit(16) {
                    None => return Err(Error::UnexpectedCharacter),
                    Some(n) => num = num * 16 + n,
                },
            }
            self.next();
        }
        Ok(num as u16)
    }

    fn parse_basic_latin_escape(&mut self) -> Result<String, Error> {
        self.next();
        self.next();

        let vec = vec![self.parse_hex_utf8()?];

        match String::from_utf8(vec) {
            Err(_) => Err(Error::UnexpectedCharacter),
            Ok(s) => Ok(s),
        }
    }

    fn parse_basic_multilingual_plane_escape(&mut self) -> Result<String, Error> {
        self.next();
        self.next();

        let mut vec = vec![self.parse_hex_utf16()?];

        // surrogate pair
        if 0xD800 <= vec[0] && vec[0] <= 0xDBFF {
            self.consume('\\')?;
            self.consume('u')?;
            vec.push(self.parse_hex_utf16()?);
        }

        match String::from_utf16(&vec) {
            Err(_) => Err(Error::UnexpectedCharacter),
            Ok(s) => Ok(s),
        }
    }

    fn parse_popular_character_escape(&mut self, c: char) -> char {
        self.next();
        self.next();

        match c {
            '\'' => '\u{0027}',
            '"' => '\u{0022}',
            '\\' => '\u{005C}',
            'b' => '\u{0008}',
            'f' => '\u{000C}',
            'n' => '\u{000A}',
            'r' => '\u{000D}',
            't' => '\u{0009}',
            'v' => '\u{000B}',
            '0' => '\u{0000}',
            _ => unreachable!(),
        }
    }

    fn skip_line_continuation(&mut self, c: char) {
        self.next();
        self.next();

        if c != '\u{000D}' {
            return;
        }
        if let Some('\u{000A}') = self.ch {
            self.next();
        }
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
