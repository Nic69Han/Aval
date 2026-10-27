//! Shared diagnostic type: one flat JSON object per diagnostic.

use serde_json::{Map, Value};

/// 1-based line and column over the whole file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos {
    pub line: u32,
    pub col: u32,
}

impl Pos {
    pub fn new(line: u32, col: u32) -> Pos {
        Pos { line, col }
    }
}

/// A diagnostic. `extra` keeps the code-specific fields in emission order.
#[derive(Clone, Debug, PartialEq)]
pub struct Diag {
    pub code: &'static str,
    pub pos: Pos,
    pub func: Option<String>,
    pub msg: String,
    pub extra: Vec<(&'static str, Value)>,
}

impl Diag {
    pub fn new(code: &'static str, pos: Pos, msg: impl Into<String>) -> Diag {
        Diag {
            code,
            pos,
            func: None,
            msg: msg.into(),
            extra: Vec::new(),
        }
    }

    pub fn with_fn(mut self, name: &str) -> Diag {
        self.func = Some(name.to_string());
        self
    }

    pub fn with(mut self, key: &'static str, value: impl Into<Value>) -> Diag {
        self.extra.push((key, value.into()));
        self
    }

    /// Errors (`E`) and self-test failures (`T`) make the exit code 1; warnings (`W`) do not.
    pub fn is_error(&self) -> bool {
        self.code.starts_with('E') || self.code.starts_with('T')
    }

    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("code".into(), Value::from(self.code));
        m.insert(
            "at".into(),
            Value::Array(vec![Value::from(self.pos.line), Value::from(self.pos.col)]),
        );
        if let Some(f) = &self.func {
            m.insert("fn".into(), Value::from(f.as_str()));
        }
        m.insert("msg".into(), Value::from(self.msg.as_str()));
        for (k, v) in &self.extra {
            m.insert((*k).into(), v.clone());
        }
        Value::Object(m)
    }

    pub fn to_line(&self) -> String {
        self.to_json().to_string()
    }
}

/// Deterministic order: by position, then code; exact duplicates removed.
pub fn sort_diags(diags: &mut Vec<Diag>) {
    diags.sort_by(|a, b| (a.pos, a.code).cmp(&(b.pos, b.code)));
    diags.dedup();
}

pub fn has_errors(diags: &[Diag]) -> bool {
    diags.iter().any(Diag::is_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_key_order() {
        let d = Diag::new("E301", Pos::new(3, 4), "type mismatch")
            .with_fn("f")
            .with("expected", "Int")
            .with("found", "Float");
        assert_eq!(
            d.to_line(),
            r#"{"code":"E301","at":[3,4],"fn":"f","msg":"type mismatch","expected":"Int","found":"Float"}"#
        );
    }

    #[test]
    fn sorting_is_by_position_then_code() {
        let mut v = vec![
            Diag::new("E101", Pos::new(5, 1), "a"),
            Diag::new("E301", Pos::new(3, 1), "b"),
            Diag::new("E101", Pos::new(5, 1), "a"),
        ];
        sort_diags(&mut v);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].code, "E301");
    }
}
