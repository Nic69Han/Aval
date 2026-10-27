//! Runtime values and their JSON encoding (design D5).

use serde_json::{Map, Number, Value as Json};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Byte(u8),
    Float(f64),
    Bool(bool),
    Str(Rc<str>),
    List(Rc<Vec<Value>>),
    Variant(Rc<str>, Rc<Vec<Value>>),
    Struct(Rc<str>, Rc<Vec<(String, Value)>>),
    Range(Rc<(Value, Value, bool)>),
    /// Lambda id and captured environment.
    Closure(Rc<(u32, Vec<(String, Value)>)>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => a == b,
            (Byte(a), Byte(b)) => a == b,
            (Float(a), Float(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Str(a), Str(b)) => a == b,
            (List(a), List(b)) => a == b,
            (Variant(n, a), Variant(m, b)) => n == m && a == b,
            (Struct(n, a), Struct(m, b)) => n == m && a == b,
            (Range(a), Range(b)) => a == b,
            _ => false,
        }
    }
}

impl Value {
    pub fn str(s: &str) -> Value {
        Value::Str(Rc::from(s))
    }

    pub fn list(v: Vec<Value>) -> Value {
        Value::List(Rc::new(v))
    }

    pub fn variant(name: &str, payload: Vec<Value>) -> Value {
        Value::Variant(Rc::from(name), Rc::new(payload))
    }

    pub fn ok(v: Value) -> Value {
        Value::variant("Ok", vec![v])
    }

    pub fn err(v: Value) -> Value {
        Value::variant("Err", vec![v])
    }

    pub fn to_json(&self) -> Json {
        match self {
            Value::Int(i) => Json::from(*i),
            Value::Byte(b) => Json::from(*b),
            Value::Float(f) => match Number::from_f64(*f) {
                Some(n) => Json::Number(n),
                None => Json::String(f.to_string()),
            },
            Value::Bool(b) => Json::Bool(*b),
            Value::Str(s) => Json::String(s.to_string()),
            Value::List(v) => Json::Array(v.iter().map(Value::to_json).collect()),
            Value::Variant(n, p) => {
                let name = n.to_string();
                if &*name == "Ok" || &*name == "Err" {
                    let mut m = Map::new();
                    m.insert(name, p.first().map(Value::to_json).unwrap_or(Json::Null));
                    Json::Object(m)
                } else if p.is_empty() {
                    Json::String(name)
                } else {
                    let mut m = Map::new();
                    m.insert(name, Json::Array(p.iter().map(Value::to_json).collect()));
                    Json::Object(m)
                }
            }
            Value::Struct(_, fields) => {
                let mut m = Map::new();
                for (k, v) in fields.iter() {
                    m.insert(k.clone(), v.to_json());
                }
                Json::Object(m)
            }
            Value::Range(r) => Json::String(format!("{}..{}", r.0.to_json(), r.1.to_json())),
            Value::Closure(_) => Json::String("<fn>".into()),
        }
    }

    /// Rough size, used to prefer the simplest failing input before shrinking.
    pub fn weight(&self) -> u64 {
        match self {
            Value::Int(i) => 1 + i.unsigned_abs().min(1_000_000),
            Value::Byte(b) => 1 + u64::from(*b),
            Value::Float(f) => {
                if f.is_finite() {
                    2 + (f.abs().min(1_000_000.0) as u64) + u64::from(f.fract() != 0.0)
                } else {
                    2_000_000
                }
            }
            Value::Bool(b) => 1 + u64::from(*b),
            Value::Str(s) => 1 + 2 * s.len() as u64,
            Value::List(v) => 1 + 4 * v.len() as u64 + v.iter().map(Value::weight).sum::<u64>(),
            Value::Variant(_, p) => 1 + p.iter().map(Value::weight).sum::<u64>(),
            Value::Struct(_, f) => 1 + f.iter().map(|(_, v)| v.weight()).sum::<u64>(),
            _ => 1,
        }
    }
}

/// `abs(a-b) <= 1e-9 * max(1, abs(a), abs(b))`, with identical values (including infinities)
/// always approximately equal.
pub fn approx(a: f64, b: f64) -> bool {
    if a == b {
        return true;
    }
    (a - b).abs() <= 1e-9 * 1.0_f64.max(a.abs()).max(b.abs())
}

/// Structural equality where every `Float` is compared with `approx`.
pub fn approx_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Float(x), Value::Float(y)) => approx(*x, *y),
        (Value::List(x), Value::List(y)) => {
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(p, q)| approx_eq(p, q))
        }
        (Value::Variant(n, x), Value::Variant(m, y)) => {
            n == m && x.len() == y.len() && x.iter().zip(y.iter()).all(|(p, q)| approx_eq(p, q))
        }
        (Value::Struct(n, x), Value::Struct(m, y)) => {
            n == m
                && x.len() == y.len()
                && x.iter()
                    .zip(y.iter())
                    .all(|((k1, p), (k2, q))| k1 == k2 && approx_eq(p, q))
        }
        _ => a == b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_encoding() {
        assert_eq!(Value::Float(2.0).to_json().to_string(), "2.0");
        assert_eq!(Value::Float(0.5).to_json().to_string(), "0.5");
        assert_eq!(
            Value::ok(Value::Int(2)).to_json().to_string(),
            r#"{"Ok":2}"#
        );
        assert_eq!(
            Value::err(Value::str("odd")).to_json().to_string(),
            r#"{"Err":"odd"}"#
        );
        assert_eq!(
            Value::variant("Red", vec![]).to_json().to_string(),
            r#""Red""#
        );
        assert_eq!(
            Value::variant("Circle", vec![Value::Float(1.0)])
                .to_json()
                .to_string(),
            r#"{"Circle":[1.0]}"#
        );
        let s = Value::Struct(
            Rc::from("P"),
            Rc::new(vec![
                ("x".into(), Value::Int(1)),
                ("y".into(), Value::Int(2)),
            ]),
        );
        assert_eq!(s.to_json().to_string(), r#"{"x":1,"y":2}"#);
        assert_eq!(
            Value::list(vec![Value::Int(1)]).to_json().to_string(),
            "[1]"
        );
    }

    #[test]
    fn approx_rules() {
        assert!(approx(0.1 + 0.2, 0.3));
        assert!(approx(f64::INFINITY, f64::INFINITY));
        assert!(!approx(1.0, 1.001));
        assert!(!approx(f64::NAN, f64::NAN));
    }
}
