//! Static types of the MVP.

use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Ty {
    Int,
    Float,
    Bool,
    Str,
    Byte,
    List(Box<Ty>),
    Result(Box<Ty>, Box<Ty>),
    User(String),
    Range(Box<Ty>),
    Fn(Vec<Ty>, Box<Ty>),
    /// Unknown (after an error, or an empty list element): unifies with everything.
    Any,
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Int => write!(f, "Int"),
            Ty::Float => write!(f, "Float"),
            Ty::Bool => write!(f, "Bool"),
            Ty::Str => write!(f, "String"),
            Ty::Byte => write!(f, "Byte"),
            Ty::List(t) => write!(f, "List<{}>", t),
            Ty::Result(t, e) => write!(f, "Result<{}, {}>", t, e),
            Ty::User(n) => write!(f, "{}", n),
            Ty::Range(t) => write!(f, "Range<{}>", t),
            Ty::Fn(ps, r) => {
                let ps: Vec<String> = ps.iter().map(|p| p.to_string()).collect();
                write!(f, "fn({}) -> {}", ps.join(", "), r)
            }
            Ty::Any => write!(f, "_"),
        }
    }
}

impl Ty {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Ty::Int | Ty::Float | Ty::Byte | Ty::Any)
    }

    pub fn is_ordered(&self) -> bool {
        matches!(self, Ty::Int | Ty::Float | Ty::Byte | Ty::Str | Ty::Any)
    }

    pub fn contains_any(&self) -> bool {
        match self {
            Ty::Any => true,
            Ty::List(t) | Ty::Range(t) => t.contains_any(),
            Ty::Result(a, b) => a.contains_any() || b.contains_any(),
            Ty::Fn(ps, r) => ps.iter().any(Ty::contains_any) || r.contains_any(),
            _ => false,
        }
    }
}

/// Structural unification where `Any` matches everything; returns the merged type.
pub fn unify(a: &Ty, b: &Ty) -> Option<Ty> {
    match (a, b) {
        (Ty::Any, x) | (x, Ty::Any) => Some(x.clone()),
        (Ty::List(x), Ty::List(y)) => unify(x, y).map(|t| Ty::List(Box::new(t))),
        (Ty::Range(x), Ty::Range(y)) => unify(x, y).map(|t| Ty::Range(Box::new(t))),
        (Ty::Result(a1, b1), Ty::Result(a2, b2)) => {
            let x = unify(a1, a2)?;
            let y = unify(b1, b2)?;
            Some(Ty::Result(Box::new(x), Box::new(y)))
        }
        (Ty::Fn(p1, r1), Ty::Fn(p2, r2)) => {
            if p1.len() != p2.len() {
                return None;
            }
            let mut ps = Vec::new();
            for (x, y) in p1.iter().zip(p2) {
                ps.push(unify(x, y)?);
            }
            Some(Ty::Fn(ps, Box::new(unify(r1, r2)?)))
        }
        (x, y) if x == y => Some(x.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        let t = Ty::Result(Box::new(Ty::List(Box::new(Ty::Float))), Box::new(Ty::Str));
        assert_eq!(t.to_string(), "Result<List<Float>, String>");
    }

    #[test]
    fn unify_any() {
        let a = Ty::List(Box::new(Ty::Any));
        let b = Ty::List(Box::new(Ty::Int));
        assert_eq!(unify(&a, &b), Some(b.clone()));
        assert_eq!(unify(&Ty::Int, &Ty::Float), None);
    }
}
