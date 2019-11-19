use std::collections::HashMap;
use std::ops::Index;

pub enum Value {
    Null,
    Boolean(bool),
    Number(i32),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, i: usize) -> &Self::Output {
        match self {
            Self::Array(vec) => vec.get(i).unwrap_or(&Self::Null),
            _ => &Self::Null,
        }
    }
}

impl Index<&str> for Value {
    type Output = Value;

    fn index(&self, key: &str) -> &Self::Output {
        match self {
            Self::Object(map) => map.get(key).unwrap_or(&Self::Null),
            _ => &Self::Null,
        }
    }
}

impl Value {
    pub fn is_null(&self) -> bool {
        match self {
            Self::Null => true,
            _ => false,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Self::Number(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_vec(&self) -> Option<&Vec<Self>> {
        match self {
            Self::Array(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, Self>> {
        match self {
            Self::Object(m) => Some(m),
            _ => None,
        }
    }
}
