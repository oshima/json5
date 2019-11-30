extern crate json5;

use json5::{parse, Value};

#[test]
fn it_works() {
    match parse("// this is a comment\nnull".to_string()) {
        Ok(Value::Null) => (),
        _ => panic!(),
    };

    match parse("-42".to_string()) {
        Ok(Value::Integer(-42)) => (),
        _ => panic!(),
    };

    assert_eq!(parse("1.23".to_string()), Ok(Value::Float(1.23)));
    assert_eq!(parse("-Infinity".to_string()), Ok(Value::Float(std::f64::NEG_INFINITY)));
    assert_eq!(parse("-2.3e2".to_string()), Ok(Value::Float(-230.0)));
    assert_eq!(parse("4.2e-2".to_string()), Ok(Value::Float(0.042)));

    match parse("\"foo bar\"".to_string()) {
        Ok(Value::String(s)) => assert_eq!(s, "foo bar".to_string()),
        _ => panic!(),
    };

    match parse("[1, true]".to_string()) {
        Ok(Value::Array(v)) => {
            assert_eq!(v.len(), 2);
            match v[0] {
                Value::Integer(1) => (),
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
                Some(Value::Integer(1)) => (),
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
