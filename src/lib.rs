use std::collections::HashMap;
use std::str::Chars;

pub enum Value {
    Null,
    False,
    True,
    Number(i32),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

struct Parser<'a> {
    chars: Chars<'a>,
    ch: char,
}

impl<'a> Parser<'a> {
    fn next(&mut self) {
        self.ch = match self.chars.next() {
            Some(ch) => ch,
            None => 0 as char,
        }
    }

    fn expect(&mut self, ch: char) -> Result<(), &'static str> {
        if self.ch != ch {
            return Err("unexpected char");
        }
        self.next();
        Ok(())
    }

    fn skip_ws(&mut self) {
        while self.ch.is_whitespace() {
            self.next();
        }
    }

    fn parse_value(mut self) -> Result<Value, &'static str> {
        &self.skip_ws();

        match self.ch {
            'n' => self.parse_null(),
            'f' => self.parse_false(),
            't' => self.parse_true(),
            '+' | '-' | '0' ..= '9' => self.parse_number(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            _ => Err("not implemented"),
        }
    }

    fn parse_null(mut self) -> Result<Value, &'static str> {
        if let Err(s) = &self.expect('n') {
            return Err(s);
        }
        if let Err(s) = &self.expect('u') {
            return Err(s);
        }
        if let Err(s) = &self.expect('l') {
            return Err(s);
        }
        if let Err(s) = &self.expect('l') {
            return Err(s);
        }
        Ok(Value::Null)
    }

    fn parse_false(mut self) -> Result<Value, &'static str> {
        if let Err(s) = &self.expect('f') {
            return Err(s);
        }
        if let Err(s) = &self.expect('a') {
            return Err(s);
        }
        if let Err(s) = &self.expect('l') {
            return Err(s);
        }
        if let Err(s) = &self.expect('s') {
            return Err(s);
        }
        if let Err(s) = &self.expect('e') {
            return Err(s);
        }
        Ok(Value::False)
    }

    fn parse_true(mut self) -> Result<Value, &'static str> {
        if let Err(s) = &self.expect('t') {
            return Err(s);
        }
        if let Err(s) = &self.expect('r') {
            return Err(s);
        }
        if let Err(s) = &self.expect('u') {
            return Err(s);
        }
        if let Err(s) = &self.expect('e') {
            return Err(s);
        }
        Ok(Value::True)
    }

    fn parse_number(mut self) -> Result<Value, &'static str> {
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

    fn parse_string(mut self) -> Result<Value, &'static str> {
        let mut s = "".to_string();

        self.next();

        while self.ch != '"' {
		    if self.ch == 0 as char {
			    return Err("unexpected eos");
		    }
            s.push(self.ch);
		    self.next();
        }
        self.next();

        Ok(Value::String(s))
    }

    fn parse_array(mut self) -> Result<Value, &'static str> {
        let mut v = vec![];

        self.next();
        self.skip_ws();

        while self.ch != ']' {
            if self.ch == 0 as char {
			    return Err("unexpected eos");
		    }
            match self.parse_value() {
                Ok(value) => v.push(value),
                Err(s) => return Err(s),
            }
            self.skip_ws();
            if self.ch == ',' {
                self.next();
                self.skip_ws();
            }
        }
        self.next();

        Ok(Value::Array(v))
    }
}

pub fn parse(json: String) -> Result<Value, &'static str> {
    let mut parser = Parser {
        chars: json.chars(),
        ch: 0 as char,
    };
    parser.next();
    parser.parse_value()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_i32(v: &Value) -> i32 {
        match v {
            Value::False => 0,
            Value::Null => 0,
            Value::True => 1,
            Value::Number(n) => *n,
            Value::String(s) => s.len() as i32,
            Value::Array(vec) => vec.iter().fold(0, |sum, x| sum + to_i32(x)),
            Value::Object(map) => map.values().fold(0, |sum, x| sum + to_i32(x)),
        }
    }

    #[test]
    fn it_works() {
        let json_number = Value::Number(4);
        let json_null = Value::Null;
        let json_string = Value::String("Hello".to_string());
        let json_array = Value::Array(vec![
            Value::True,
            Value::Number(2),
            Value::String("three".to_string()),
            Value::Array(vec![Value::Number(5), Value::Number(5)]),
        ]);
        let json_object = Value::Object({
            let mut map = HashMap::new();
            map.insert("foo".to_string(), Value::Number(10));
            map.insert("bar".to_string(), Value::Number(20));
            map
        });

        assert_eq!(to_i32(&json_number), 4);
        assert_eq!(to_i32(&json_null), 0);
        assert_eq!(to_i32(&json_string), 5);
        assert_eq!(to_i32(&json_array), 18);
        assert_eq!(to_i32(&json_object), 30);

        match parse("null".to_string()) {
            Ok(Value::Null) => (),
            _ => panic!(),
        }

        match parse("-42".to_string()) {
            Ok(Value::Number(n)) => assert_eq!(n, -42),
            _ => panic!(),
        }

       match parse("\"foo bar\"".to_string()) {
            Ok(Value::String(s)) => assert_eq!(s, "foo bar".to_string()),
            _ => panic!(),
        }
    }
}
