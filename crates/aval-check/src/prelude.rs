//! The prelude table (spec section 9): one source for the checker, the interpreter and the
//! `avail` list of `E204`.

use crate::info::ModuleInfo;
use crate::ty::Ty;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recv {
    /// Any `List<T>`.
    List,
    /// `List<Int>` or `List<Float>`.
    ListNumeric,
    /// `List<T>` with `T` ordered.
    ListOrdered,
    /// `List<String>`.
    ListStr,
    /// `Range<Int>`.
    Range,
    Int,
    Float,
    Byte,
    Bool,
    Str,
    Result,
}

pub struct Method {
    pub name: &'static str,
    pub recv: Recv,
    pub arity: usize,
    pub cost: &'static str,
}

const fn m(name: &'static str, recv: Recv, arity: usize, cost: &'static str) -> Method {
    Method {
        name,
        recv,
        arity,
        cost,
    }
}

pub const METHODS: &[Method] = &[
    m("len", Recv::List, 0, "O(1)"),
    m("sum", Recv::ListNumeric, 0, "O(n)"),
    m("min", Recv::ListOrdered, 0, "O(n)"),
    m("max", Recv::ListOrdered, 0, "O(n)"),
    m("contains", Recv::List, 1, "O(n)"),
    m("map", Recv::List, 1, "O(n * cost(f))"),
    m("filter", Recv::List, 1, "O(n * cost(f))"),
    m("fold", Recv::List, 2, "O(n * cost(f))"),
    m("count", Recv::List, 1, "O(n)"),
    m("reverse", Recv::List, 0, "O(n)"),
    m("sort", Recv::ListOrdered, 0, "O(n log n)"),
    m("push", Recv::List, 1, "O(n)"),
    m("float", Recv::Int, 0, "O(1)"),
    m("abs", Recv::Int, 0, "O(1)"),
    m("float", Recv::Byte, 0, "O(1)"),
    m("int", Recv::Byte, 0, "O(1)"),
    m("abs", Recv::Float, 0, "O(1)"),
    m("len", Recv::Str, 0, "O(1)"),
    m("is_ok", Recv::Result, 0, "O(1)"),
    m("is_err", Recv::Result, 0, "O(1)"),
    m("unwrap", Recv::Result, 0, "O(1)"),
    m("is_empty", Recv::List, 0, "O(1)"),
    m("any", Recv::List, 1, "O(n * cost(f))"),
    m("all", Recv::List, 1, "O(n * cost(f))"),
    m("take", Recv::List, 1, "O(n)"),
    m("skip", Recv::List, 1, "O(n)"),
    m("concat", Recv::List, 1, "O(n)"),
    m("dedup", Recv::List, 0, "O(n)"),
    m("product", Recv::ListNumeric, 0, "O(n)"),
    m("join", Recv::ListStr, 1, "O(n)"),
    m("min", Recv::Int, 1, "O(1)"),
    m("max", Recv::Int, 1, "O(1)"),
    m("pow", Recv::Int, 1, "O(log e)"),
    m("to_string", Recv::Int, 0, "O(1)"),
    m("sqrt", Recv::Float, 0, "O(1)"),
    m("floor", Recv::Float, 0, "O(1)"),
    m("ceil", Recv::Float, 0, "O(1)"),
    m("round", Recv::Float, 0, "O(1)"),
    m("min", Recv::Float, 1, "O(1)"),
    m("max", Recv::Float, 1, "O(1)"),
    m("int", Recv::Float, 0, "O(1)"),
    m("to_string", Recv::Float, 0, "O(1)"),
    m("to_string", Recv::Byte, 0, "O(1)"),
    m("to_string", Recv::Bool, 0, "O(1)"),
    m("to_string", Recv::Str, 0, "O(n)"),
    m("is_empty", Recv::Str, 0, "O(1)"),
    m("chars", Recv::Str, 0, "O(n)"),
    m("contains", Recv::Str, 1, "O(n)"),
    m("starts_with", Recv::Str, 1, "O(n)"),
    m("ends_with", Recv::Str, 1, "O(n)"),
    m("split", Recv::Str, 1, "O(n)"),
    m("trim", Recv::Str, 0, "O(n)"),
    m("to_uppercase", Recv::Str, 0, "O(n)"),
    m("to_lowercase", Recv::Str, 0, "O(n)"),
    m("parse_int", Recv::Str, 0, "O(n)"),
    m("len", Recv::Range, 0, "O(1)"),
    m("is_empty", Recv::Range, 0, "O(1)"),
    m("contains", Recv::Range, 1, "O(1)"),
    m("sum", Recv::Range, 0, "O(n)"),
    m("product", Recv::Range, 0, "O(n)"),
    m("min", Recv::Range, 0, "O(n)"),
    m("max", Recv::Range, 0, "O(n)"),
    m("count", Recv::Range, 1, "O(n)"),
    m("map", Recv::Range, 1, "O(n * cost(f))"),
    m("filter", Recv::Range, 1, "O(n * cost(f))"),
    m("fold", Recv::Range, 2, "O(n * cost(f))"),
    m("any", Recv::Range, 1, "O(n * cost(f))"),
    m("all", Recv::Range, 1, "O(n * cost(f))"),
    m("reverse", Recv::Range, 0, "O(n)"),
    m("take", Recv::Range, 1, "O(n)"),
    m("skip", Recv::Range, 1, "O(n)"),
];

fn recv_ok(r: Recv, ty: &Ty) -> bool {
    match ty {
        Ty::List(t) => match r {
            Recv::List => true,
            Recv::ListNumeric => matches!(**t, Ty::Int | Ty::Float | Ty::Any),
            Recv::ListOrdered => t.is_ordered(),
            Recv::ListStr => matches!(**t, Ty::Str | Ty::Any),
            _ => false,
        },
        Ty::Range(t) => r == Recv::Range && matches!(**t, Ty::Int | Ty::Any),
        Ty::Int => r == Recv::Int,
        Ty::Float => r == Recv::Float,
        Ty::Byte => r == Recv::Byte,
        Ty::Bool => r == Recv::Bool,
        Ty::Str => r == Recv::Str,
        Ty::Result(..) => r == Recv::Result,
        _ => false,
    }
}

/// The prelude method `name` on a receiver of type `ty`, if any.
pub fn lookup(ty: &Ty, name: &str) -> Option<&'static Method> {
    METHODS
        .iter()
        .find(|me| me.name == name && recv_ok(me.recv, ty))
}

/// Free prelude functions: (name, arity, effects).
pub const FUNCTIONS: &[(&str, usize, &[&str])] =
    &[("approx", 2, &[]), ("print", 1, &["io.stdout"])];

pub fn function(name: &str) -> Option<(usize, &'static [&'static str])> {
    FUNCTIONS
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|(_, a, e)| (*a, *e))
}

/// Methods available on a receiver type, sorted; for structs the field names.
pub fn avail(ty: &Ty, info: &ModuleInfo) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    match ty {
        Ty::User(n) => {
            if let Some(fields) = info.structs.get(n) {
                out.extend(fields.iter().map(|(f, _)| f.clone()));
            }
        }
        _ => {
            for me in METHODS {
                if recv_ok(me.recv, ty) {
                    out.push(me.name.to_string());
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Cost declaration of a method, if any (used by the skill reference).
pub fn cost_of(name: &str) -> Option<&'static str> {
    METHODS.iter().find(|m| m.name == name).map(|m| m.cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_float_avail_is_sorted_and_has_basics() {
        let info = ModuleInfo::default();
        let a = avail(&Ty::List(Box::new(Ty::Float)), &info);
        for m in ["len", "sum", "min", "max"] {
            assert!(a.contains(&m.to_string()));
        }
        let mut s = a.clone();
        s.sort();
        assert_eq!(a, s);
    }

    #[test]
    fn range_avail_lists_methods() {
        let info = ModuleInfo::default();
        let a = avail(&Ty::Range(Box::new(Ty::Int)), &info);
        for m in ["fold", "sum", "len", "map", "contains", "skip"] {
            assert!(a.contains(&m.to_string()), "{m}");
        }
        assert!(avail(&Ty::Range(Box::new(Ty::Float)), &info).is_empty());
    }

    #[test]
    fn bool_list_has_no_sum() {
        let info = ModuleInfo::default();
        let a = avail(&Ty::List(Box::new(Ty::Bool)), &info);
        assert!(!a.contains(&"sum".to_string()));
        assert!(a.contains(&"len".to_string()));
    }

    #[test]
    fn print_has_io_stdout() {
        assert_eq!(function("print").unwrap().1, &["io.stdout"]);
        assert!(function("approx").unwrap().1.is_empty());
    }
}
