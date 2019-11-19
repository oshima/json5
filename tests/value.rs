extern crate json5;

use json5::Value;
use std::collections::HashMap;

#[test]
fn it_works() {
    fn to_i32(v: &Value) -> i32 {
        match v {
            Value::Null => 0,
            Value::Boolean(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            Value::Number(n) => *n,
            Value::String(s) => s.len() as i32,
            Value::Array(vec) => vec.iter().fold(0, |sum, x| sum + to_i32(x)),
            Value::Object(map) => map.values().fold(0, |sum, x| sum + to_i32(x)),
        }
    }

    let null = Value::Null;
    let number = Value::Number(4);
    let string = Value::String("Hello".to_string());
    let array = Value::Array(vec![
        Value::Boolean(true),
        Value::Number(2),
        Value::String("three".to_string()),
        Value::Array(vec![Value::Number(5), Value::Number(5)]),
    ]);
    let object = Value::Object({
        let mut map = HashMap::new();
        map.insert("foo".to_string(), Value::Number(10));
        map.insert("bar".to_string(), Value::Number(20));
        map
    });

    assert_eq!(to_i32(&null), 0);
    assert_eq!(to_i32(&number), 4);
    assert_eq!(to_i32(&string), 5);
    assert_eq!(to_i32(&array), 18);
    assert_eq!(to_i32(&object), 30);
}

#[test]
fn index() {
    let null = Value::Null;
    let number = Value::Number(4);
    let array = Value::Array(vec![
        Value::Boolean(true),
        Value::Number(2),
        Value::String("three".to_string()),
        Value::Array(vec![Value::Number(5), Value::Number(5)]),
    ]);
    let object = Value::Object({
        let mut map = HashMap::new();
        map.insert("foo".to_string(), Value::Number(10));
        map.insert("bar".to_string(), Value::Array(vec![
            Value::Number(20),
            Value::Number(30),
        ]));
        map
    });

    assert!(null[0].is_null());
    assert!(number[0].is_null());
    assert!(array[0].as_bool().unwrap() == true);
    assert!(array[3][1].as_i32().unwrap() == 5);
    assert!(object[0].is_null());
    assert!(object["foo"].as_i32().unwrap() == 10);
    assert!(object["bar"][1].as_i32().unwrap() == 30);
    assert!(object["bar"][1].as_string().is_none());
}

#[test]
fn as_i32() {
    assert!(Value::Number(-3).as_i32().unwrap() == -3);
    assert!(Value::Boolean(true).as_i32().is_none());
}
