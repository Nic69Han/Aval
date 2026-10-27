//! Module-level declarations: types, enums, variants and function signatures.

use crate::ty::Ty;
use aval_syntax::ast::*;
use aval_syntax::diag::{Diag, Pos};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct FnSig {
    pub params: Vec<(String, Ty)>,
    pub ret: Ty,
    pub effects: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ModuleInfo {
    pub module_name: String,
    pub structs: BTreeMap<String, Vec<(String, Ty)>>,
    pub enums: BTreeMap<String, Vec<(String, Vec<Ty>)>>,
    /// variant name -> enum name
    pub variants: BTreeMap<String, String>,
    /// first declaration of each function name
    pub fns: BTreeMap<String, FnSig>,
    /// signature of every function item, by item index
    pub sig_by_item: BTreeMap<usize, FnSig>,
    /// ids of integer literals typed `Byte`
    pub byte_lits: BTreeSet<u32>,
    /// ids of `.sum()` calls on `List<Float>`
    pub float_sums: BTreeSet<u32>,
    /// items that failed to parse
    pub broken: BTreeSet<String>,
    /// construction depth of every inhabited user type (see `compute_ranks`)
    pub ranks: BTreeMap<String, u32>,
}

impl ModuleInfo {
    pub fn type_names(&self) -> BTreeSet<String> {
        self.structs
            .keys()
            .chain(self.enums.keys())
            .cloned()
            .collect()
    }

    /// Variants of an enum or of `Result`.
    pub fn ctors(&self, ty: &Ty) -> Option<Vec<(String, Vec<Ty>)>> {
        match ty {
            Ty::User(n) => self.enums.get(n).cloned(),
            Ty::Result(t, e) => Some(vec![
                ("Ok".into(), vec![(**t).clone()]),
                ("Err".into(), vec![(**e).clone()]),
            ]),
            _ => None,
        }
    }
}

/// Resolve a written type. Unknown or unsupported names report `E101`.
pub fn resolve_type(
    te: &TypeExpr,
    types: &BTreeSet<String>,
    broken: &BTreeSet<String>,
    diags: &mut Vec<Diag>,
    func: Option<&str>,
) -> Ty {
    fn push(diags: &mut Vec<Diag>, func: Option<&str>, d: Diag) {
        diags.push(match func {
            Some(f) => d.with_fn(f),
            None => d,
        })
    }
    let type_args: Vec<&TypeExpr> = te
        .args
        .iter()
        .filter_map(|a| match a {
            TypeArg::Type(t) => Some(t),
            _ => None,
        })
        .collect();
    let simple = |n: usize| te.args.len() == n && type_args.len() == n;
    match te.name.as_str() {
        "Int" | "Float" | "Bool" | "String" | "Byte" if !te.args.is_empty() => {
            push(
                diags,
                func,
                Diag::new("E303", te.pos, "type takes no arguments")
                    .with("expected", 0)
                    .with("found", te.args.len()),
            );
            Ty::Any
        }
        "Int" => Ty::Int,
        "Float" => Ty::Float,
        "Bool" => Ty::Bool,
        "String" => Ty::Str,
        "Byte" => Ty::Byte,
        "List" | "Result" => {
            let n = if te.name == "List" { 1 } else { 2 };
            if !simple(n) {
                push(
                    diags,
                    func,
                    Diag::new("E303", te.pos, "wrong number of type arguments")
                        .with("expected", n)
                        .with("found", te.args.len()),
                );
                return Ty::Any;
            }
            let args: Vec<Ty> = type_args
                .iter()
                .map(|t| resolve_type(t, types, broken, diags, func))
                .collect();
            if n == 1 {
                Ty::List(Box::new(args[0].clone()))
            } else {
                Ty::Result(Box::new(args[0].clone()), Box::new(args[1].clone()))
            }
        }
        name if broken.contains(name) => Ty::Any,
        name if types.contains(name) => {
            if !te.args.is_empty() {
                push(
                    diags,
                    func,
                    Diag::new("E303", te.pos, "type takes no arguments")
                        .with("expected", 0)
                        .with("found", te.args.len()),
                );
            }
            Ty::User(name.to_string())
        }
        "Set" | "Map" | "Queue" | "Image" | "Tensor" | "Arena" => {
            push(
                diags,
                func,
                Diag::new("E101", te.pos, "type not supported in the MVP")
                    .with("name", te.name.as_str()),
            );
            Ty::Any
        }
        name => {
            push(
                diags,
                func,
                Diag::new("E101", te.pos, "unknown type").with("name", name),
            );
            Ty::Any
        }
    }
}

pub fn build(module: &Module, diags: &mut Vec<Diag>) -> ModuleInfo {
    let mut info = ModuleInfo {
        module_name: module.name.clone(),
        ..Default::default()
    };
    let broken: BTreeSet<String> = module.broken.iter().cloned().collect();
    info.broken = broken.clone();
    for im in &module.imports {
        diags.push(
            Diag::new(
                "E103",
                im.pos,
                "unresolved module (imports are not supported in the MVP)",
            )
            .with("module", im.path.as_str()),
        );
    }
    // pass 1: names
    let mut seen: BTreeMap<String, Pos> = BTreeMap::new();
    let mut types: BTreeSet<String> = BTreeSet::new();
    for it in &module.items {
        let (name, is_type) = match &it.kind {
            ItemKind::Fn(f) => (f.name.clone(), false),
            ItemKind::Type(t) | ItemKind::Resource(t) => (t.name.clone(), true),
            ItemKind::Enum(e) => (e.name.clone(), true),
        };
        if is_type && name == "O" {
            diags.push(
                Diag::new("E105", it.pos, "`O` is reserved for `cost O(...)`")
                    .with("name", name.as_str()),
            );
            continue;
        }
        if matches!(name.as_str(), "approx" | "print" | "Ok" | "Err") {
            let mut d = Diag::new("E102", it.pos, "name is reserved by the prelude")
                .with("name", name.as_str());
            if let ItemKind::Fn(_) = &it.kind {
                d = d.with_fn(&name);
            }
            diags.push(d);
            continue;
        }
        if seen.contains_key(&name) {
            let mut d =
                Diag::new("E102", it.pos, "duplicate item name").with("name", name.as_str());
            if let ItemKind::Fn(_) = &it.kind {
                d = d.with_fn(&name);
            }
            diags.push(d);
            continue;
        }
        seen.insert(name.clone(), it.pos);
        if is_type {
            types.insert(name);
        }
    }
    // pass 2: struct fields, enum variants
    for it in &module.items {
        match &it.kind {
            ItemKind::Type(t) | ItemKind::Resource(t) => {
                if !types.contains(&t.name) || info.structs.contains_key(&t.name) {
                    continue;
                }
                let mut fields: Vec<(String, Ty)> = Vec::new();
                for f in &t.fields {
                    if fields.iter().any(|(n, _)| *n == f.name) {
                        diags.push(
                            Diag::new("E102", f.pos, "duplicate field name")
                                .with("name", f.name.as_str()),
                        );
                        continue;
                    }
                    let ty = resolve_type(&f.ty, &types, &broken, diags, None);
                    fields.push((f.name.clone(), ty));
                }
                info.structs.insert(t.name.clone(), fields);
            }
            ItemKind::Enum(e) => {
                if !types.contains(&e.name)
                    || info.enums.contains_key(&e.name)
                    || info.structs.contains_key(&e.name)
                {
                    continue;
                }
                let mut vs: Vec<(String, Vec<Ty>)> = Vec::new();
                for v in &e.variants {
                    let dup = vs.iter().any(|(n, _)| *n == v.name)
                        || info.variants.contains_key(&v.name)
                        || v.name == "Ok"
                        || v.name == "Err"
                        || types.contains(&v.name);
                    if dup {
                        diags.push(
                            Diag::new("E102", v.pos, "duplicate variant name")
                                .with("name", v.name.as_str()),
                        );
                        continue;
                    }
                    let tys = v
                        .payload
                        .iter()
                        .map(|t| resolve_type(t, &types, &broken, diags, None))
                        .collect();
                    info.variants.insert(v.name.clone(), e.name.clone());
                    vs.push((v.name.clone(), tys));
                }
                info.enums.insert(e.name.clone(), vs);
            }
            ItemKind::Fn(_) => {}
        }
    }
    // pass 2b: every declared type must have a finite value
    info.ranks = compute_ranks(&info.structs, &info.enums);
    for it in &module.items {
        let name = match &it.kind {
            ItemKind::Type(t) | ItemKind::Resource(t) => &t.name,
            ItemKind::Enum(e) => &e.name,
            ItemKind::Fn(_) => continue,
        };
        let declared = info.structs.contains_key(name) || info.enums.contains_key(name);
        if declared && !info.ranks.contains_key(name) {
            diags.push(
                Diag::new(
                    "E312",
                    it.pos,
                    "type has no finite value (every construction recurses)",
                )
                .with("name", name.as_str()),
            );
        }
    }
    // pass 3: function signatures
    for (idx, it) in module.items.iter().enumerate() {
        if let ItemKind::Fn(f) = &it.kind {
            let first = !info.fns.contains_key(&f.name);
            let mut sink = Vec::new();
            let target: &mut Vec<Diag> = if first { diags } else { &mut sink };
            let params = f
                .params
                .iter()
                .map(|p| {
                    (
                        p.name.clone(),
                        resolve_type(&p.ty, &types, &broken, target, Some(&f.name)),
                    )
                })
                .collect();
            let ret = resolve_type(&f.ret, &types, &broken, target, Some(&f.name));
            let effects = f
                .effects
                .as_ref()
                .map(|e| e.names.iter().map(|(n, _)| n.clone()).collect())
                .unwrap_or_default();
            let sig = FnSig {
                params,
                ret,
                effects,
            };
            if first {
                info.fns.insert(f.name.clone(), sig.clone());
            }
            info.sig_by_item.insert(idx, sig);
        }
    }
    info
}

/// Construction depth of a type: `None` when it has no finite value.
pub fn rank_of(ty: &Ty, ranks: &BTreeMap<String, u32>) -> Option<u32> {
    match ty {
        Ty::User(n) => ranks.get(n).copied(),
        Ty::Result(a, b) => match (rank_of(a, ranks), rank_of(b, ranks)) {
            (Some(x), Some(y)) => Some(x.min(y)),
            (x, y) => x.or(y),
        },
        _ => Some(0),
    }
}

/// Least fixed point: a struct needs all fields, an enum needs one variant with all payloads.
pub fn compute_ranks(
    structs: &BTreeMap<String, Vec<(String, Ty)>>,
    enums: &BTreeMap<String, Vec<(String, Vec<Ty>)>>,
) -> BTreeMap<String, u32> {
    let mut ranks: BTreeMap<String, u32> = BTreeMap::new();
    loop {
        let mut changed = false;
        for (n, fields) in structs {
            if ranks.contains_key(n) {
                continue;
            }
            let rs: Option<Vec<u32>> = fields.iter().map(|(_, t)| rank_of(t, &ranks)).collect();
            if let Some(rs) = rs {
                ranks.insert(n.clone(), 1 + rs.into_iter().max().unwrap_or(0));
                changed = true;
            }
        }
        for (n, variants) in enums {
            if ranks.contains_key(n) {
                continue;
            }
            let best = variants
                .iter()
                .filter_map(|(_, p)| {
                    let rs: Option<Vec<u32>> = p.iter().map(|t| rank_of(t, &ranks)).collect();
                    rs.map(|r| 1 + r.into_iter().max().unwrap_or(0))
                })
                .min();
            if let Some(b) = best {
                ranks.insert(n.clone(), b);
                changed = true;
            }
        }
        if !changed {
            return ranks;
        }
    }
}
