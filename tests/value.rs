extern crate json5;

use json5::Value;
use std::collections::HashMap;

#[test]
fn index() {
    let null = Value::Null;
    let integer = Value::Integer(4);
    let array = Value::Array(vec![
        Value::Boolean(true),
        Value::Integer(2),
        Value::String("three".to_string()),
        Value::Array(vec![Value::Integer(5), Value::Integer(5)]),
    ]);
    let object = Value::Object({
        let mut map = HashMap::new();
        map.insert("foo".to_string(), Value::Integer(10));
        map.insert(
            "bar".to_string(),
            Value::Array(vec![Value::Integer(20), Value::Integer(30)]),
        );
        map
    });

    assert!(null[0].is_null());
    assert!(integer[0].is_null());
    assert!(array[0].as_bool().unwrap() == true);
    assert!(array[3][1].as_i32().unwrap() == 5);
    assert!(object[0].is_null());
    assert!(object["foo"].as_i32().unwrap() == 10);
    assert!(object["bar"][1].as_i32().unwrap() == 30);
    assert!(object["bar"][1].as_string().is_none());
}

#[test]
fn as_i32() {
    assert!(Value::Integer(-3).as_i32().unwrap() == -3);
    assert!(Value::Boolean(true).as_i32().is_none());
}

#[test]
fn as_f64() {
    assert!(Value::Float(1.23).as_f64().unwrap() == 1.23);
}
