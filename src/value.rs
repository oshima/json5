use std::collections::HashMap;
use std::ops::Index;

#[derive(Debug)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i32),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Null => match other {
                Self::Null => true,
                _ => false,
            },
            Self::Boolean(b1) => match other {
                Self::Boolean(b2) => b1 == b2,
                _ => false,
            },
            Self::Integer(i1) => match other {
                Self::Integer(i2) => i1 == i2,
                _ => false,
            },
            Self::Float(f1) => match other {
                Self::Float(f2) => f1 == f2,
                _ => false,
            },
            Self::String(s1) => match other {
                Self::String(s2) => s1 == s2,
                _ => false,
            },
            Self::Array(v1) => match other {
                Self::Array(v2) => v1 == v2,
                _ => false,
            },
            Self::Object(m1) => match other {
                Self::Object(m2) => m1 == m2,
                _ => false,
            },
        }
    }
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

    pub fn to_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn to_i32(&self) -> Option<i32> {
        match self {
            Self::Integer(i) => Some(*i),
            Self::Float(f) => Some(*f as i32),
            _ => None,
        }
    }

    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Self::Integer(i) => Some(*i as f64),
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn to_string(&self) -> Option<&String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn to_vec(&self) -> Option<&Vec<Self>> {
        match self {
            Self::Array(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_map(&self) -> Option<&HashMap<String, Self>> {
        match self {
            Self::Object(m) => Some(m),
            _ => None,
        }
    }
}
