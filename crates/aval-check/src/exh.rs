//! Match exhaustiveness: Maranget's usefulness algorithm on flat/nested constructor patterns.

use crate::info::ModuleInfo;
use crate::ty::Ty;

#[derive(Clone, Debug, PartialEq)]
pub enum Pat {
    Wild,
    /// A literal of an infinite type (Int, Float, String, Byte).
    Lit(String),
    Ctor(String, Vec<Pat>),
}

fn head_ctors(ty: &Ty, info: &ModuleInfo) -> Option<Vec<(String, Vec<Ty>)>> {
    match ty {
        Ty::Bool => Some(vec![("true".into(), vec![]), ("false".into(), vec![])]),
        other => info.ctors(other),
    }
}

fn specialize(rows: &[Vec<Pat>], name: &str, arity: usize, lit: bool) -> Vec<Vec<Pat>> {
    let mut out = Vec::new();
    for r in rows {
        match &r[0] {
            Pat::Wild => {
                let mut n = vec![Pat::Wild; arity];
                n.extend_from_slice(&r[1..]);
                out.push(n);
            }
            Pat::Ctor(c, subs) if !lit && c == name => {
                let mut n = subs.clone();
                n.extend_from_slice(&r[1..]);
                out.push(n);
            }
            Pat::Lit(l) if lit && l == name => out.push(r[1..].to_vec()),
            _ => {}
        }
    }
    out
}

pub fn useful(info: &ModuleInfo, rows: &[Vec<Pat>], q: &[Pat], tys: &[Ty]) -> bool {
    if q.is_empty() {
        return rows.is_empty();
    }
    let t = &tys[0];
    match &q[0] {
        Pat::Ctor(c, subs) => {
            let ctys = head_ctors(t, info)
                .and_then(|cs| cs.into_iter().find(|(n, _)| n == c).map(|(_, t)| t))
                .unwrap_or_else(|| vec![Ty::Any; subs.len()]);
            let srows = specialize(rows, c, subs.len(), false);
            let mut sq = subs.clone();
            sq.extend_from_slice(&q[1..]);
            let mut stys = ctys;
            stys.extend_from_slice(&tys[1..]);
            useful(info, &srows, &sq, &stys)
        }
        Pat::Lit(l) => {
            let srows = specialize(rows, l, 0, true);
            useful(info, &srows, &q[1..], &tys[1..])
        }
        Pat::Wild => {
            if let Some(all) = head_ctors(t, info) {
                let used: Vec<&String> = rows
                    .iter()
                    .filter_map(|r| match &r[0] {
                        Pat::Ctor(c, _) => Some(c),
                        _ => None,
                    })
                    .collect();
                let complete = !all.is_empty() && all.iter().all(|(n, _)| used.contains(&n));
                if complete {
                    for (c, ctys) in all {
                        let srows = specialize(rows, &c, ctys.len(), false);
                        let mut sq = vec![Pat::Wild; ctys.len()];
                        sq.extend_from_slice(&q[1..]);
                        let mut stys = ctys;
                        stys.extend_from_slice(&tys[1..]);
                        if useful(info, &srows, &sq, &stys) {
                            return true;
                        }
                    }
                    return false;
                }
            }
            let drows: Vec<Vec<Pat>> = rows
                .iter()
                .filter(|r| r[0] == Pat::Wild)
                .map(|r| r[1..].to_vec())
                .collect();
            useful(info, &drows, &q[1..], &tys[1..])
        }
    }
}

/// Top-level constructors (or `_`) that the arms do not cover; empty when exhaustive.
pub fn missing(info: &ModuleInfo, arms: &[Pat], ty: &Ty) -> Vec<String> {
    let rows: Vec<Vec<Pat>> = arms.iter().map(|p| vec![p.clone()]).collect();
    let tys = [ty.clone()];
    if let Some(all) = head_ctors(ty, info) {
        let mut out = Vec::new();
        for (c, ctys) in all {
            let q = [Pat::Ctor(c.clone(), vec![Pat::Wild; ctys.len()])];
            if useful(info, &rows, &q, &tys) {
                out.push(c);
            }
        }
        out
    } else if useful(info, &rows, &[Pat::Wild], &tys) {
        vec!["_".to_string()]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn color_info() -> ModuleInfo {
        let mut info = ModuleInfo::default();
        info.enums.insert(
            "Color".into(),
            vec![
                ("Red".into(), vec![]),
                ("Green".into(), vec![]),
                ("Blue".into(), vec![]),
            ],
        );
        info
    }

    fn c(n: &str) -> Pat {
        Pat::Ctor(n.into(), vec![])
    }

    #[test]
    fn missing_variant_is_listed() {
        let info = color_info();
        let m = missing(&info, &[c("Red"), c("Green")], &Ty::User("Color".into()));
        assert_eq!(m, vec!["Blue"]);
        assert!(missing(&info, &[c("Red"), Pat::Wild], &Ty::User("Color".into())).is_empty());
    }

    #[test]
    fn nested_patterns_and_result() {
        let info = color_info();
        let ty = Ty::Result(Box::new(Ty::User("Color".into())), Box::new(Ty::Str));
        let arms = [
            Pat::Ctor("Ok".into(), vec![c("Red")]),
            Pat::Ctor("Err".into(), vec![Pat::Wild]),
        ];
        assert_eq!(missing(&info, &arms, &ty), vec!["Ok"]);
    }

    #[test]
    fn literals_need_wildcard() {
        let info = ModuleInfo::default();
        assert_eq!(missing(&info, &[Pat::Lit("1".into())], &Ty::Int), vec!["_"]);
        assert!(missing(&info, &[Pat::Lit("1".into()), Pat::Wild], &Ty::Int).is_empty());
        let both = [c("true"), c("false")];
        assert!(missing(&info, &both, &Ty::Bool).is_empty());
        assert_eq!(missing(&info, &[c("true")], &Ty::Bool), vec!["false"]);
    }
}
