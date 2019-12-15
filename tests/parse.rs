extern crate json5;

use json5::{parse, Error, Value};
use std::collections::HashMap;

#[test]
fn it_works() {
    assert_eq!(parse("null"), Ok(Value::Null));

    assert_eq!(parse("true"), Ok(Value::Boolean(true)));
    assert_eq!(parse("false"), Ok(Value::Boolean(false)));

    assert_eq!(parse("0"), Ok(Value::Integer(0)));
    assert_eq!(parse("00"), Err(Error::UnparseableNumber));
    assert_eq!(parse("42"), Ok(Value::Integer(42)));
    assert_eq!(parse("+42"), Ok(Value::Integer(42)));
    assert_eq!(parse("++42"), Err(Error::UnparseableNumber));
    assert_eq!(parse("-999"), Ok(Value::Integer(-999)));
    assert_eq!(parse("0x1a"), Ok(Value::Integer(26)));
    assert_eq!(parse("0X1A"), Ok(Value::Integer(26)));
    assert_eq!(parse("-0x0f"), Ok(Value::Integer(-15)));

    assert_eq!(parse("0.0"), Ok(Value::Float(0.0)));
    assert_eq!(parse("0."), Ok(Value::Float(0.0)));
    assert_eq!(parse(".0"), Ok(Value::Float(0.0)));
    assert_eq!(parse("12.3"), Ok(Value::Float(12.3)));
    assert_eq!(parse("1.23e1"), Ok(Value::Float(12.3)));
    assert_eq!(parse("1.23e+1"), Ok(Value::Float(12.3)));
    assert_eq!(parse("123e-1"), Ok(Value::Float(12.3)));
    assert_eq!(parse("1.23E1"), Ok(Value::Float(12.3)));
    assert_eq!(parse("-.33"), Ok(Value::Float(-0.33)));
    assert_eq!(parse("-9.9e2"), Ok(Value::Float(-990.0)));
    assert_eq!(parse("Infinity"), Ok(Value::Float(std::f64::INFINITY)));
    assert_eq!(parse("+Infinity"), Ok(Value::Float(std::f64::INFINITY)));
    assert_eq!(parse("-Infinity"), Ok(Value::Float(std::f64::NEG_INFINITY)));
    assert_eq!(parse("NaN").unwrap().to_f64().unwrap().is_nan(), true);
    assert_eq!(parse("+NaN").unwrap().to_f64().unwrap().is_nan(), true);
    assert_eq!(parse("-NaN").unwrap().to_f64().unwrap().is_nan(), true);

    assert_eq!(
        parse("\"foo bar\""),
        Ok(Value::String("foo bar".to_string()))
    );
    assert_eq!(
        parse("\"こんにちは😁\""),
        Ok(Value::String("こんにちは😁".to_string()))
    );

    assert_eq!(
        parse("[1, true]"),
        Ok(Value::Array(vec![Value::Integer(1), Value::Boolean(true)])),
    );

    assert_eq!(
        parse(
            r#"
            /* comment 1 is a
               multi-line comment */
            {
                // comment 2
                "foo": 1, // comment 3
                "bar": true,
            }
            // comment 4
            "#
        ),
        Ok(Value::Object({
            let mut m = HashMap::new();
            m.insert("foo".to_string(), Value::Integer(1));
            m.insert("bar".to_string(), Value::Boolean(true));
            m
        })),
    );
}
