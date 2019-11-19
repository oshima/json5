extern crate json5;

use json5::{parse, Value};

#[test]
fn it_works() {
    match parse("// this is a comment\nnull".to_string()) {
        Ok(Value::Null) => (),
        _ => panic!(),
    };

    match parse("-42".to_string()) {
        Ok(Value::Number(-42)) => (),
        _ => panic!(),
    };

    match parse("\"foo bar\"".to_string()) {
        Ok(Value::String(s)) => assert_eq!(s, "foo bar".to_string()),
        _ => panic!(),
    };

    match parse("[1, true]".to_string()) {
        Ok(Value::Array(v)) => {
            assert_eq!(v.len(), 2);
            match v[0] {
                Value::Number(1) => (),
                _ => panic!(),
            }
            match v[1] {
                Value::Boolean(true) => (),
                _ => panic!(),
            }
        }
        _ => panic!(),
    };

    let json = r#"
    /* comment 1 is a
       multi-line comment */
    {
        // comment 2
        "foo": 1, // comment 3
        "bar": true,
    }
    // comment 4
    "#;
    match parse(json.to_string()) {
        Ok(Value::Object(m)) => {
            assert_eq!(m.len(), 2);
            match m.get("foo") {
                Some(Value::Number(1)) => (),
                _ => panic!(),
            }
            match m.get("bar") {
                Some(Value::Boolean(true)) => (),
                _ => panic!(),
            }
        }
        _ => panic!(),
    };
}
