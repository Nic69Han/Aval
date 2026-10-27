//! Spec lock: canonical hashes of `pub` function specs and the `aval.lock` format.

use aval_syntax::ast::*;
use aval_syntax::diag::Diag;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

fn callees(f: &FnDecl, module: &Module) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut seen = |e: &Expr| {
        if let ExprKind::Call(n, _) = &e.kind {
            if module.fns().any(|(_, g)| g.name == *n) {
                names.push(n.clone());
            }
        }
    };
    f.walk_all(&mut seen);
    if let Some((o, _)) = &f.oracle {
        names.push(o.clone());
    }
    names
}

/// Module functions reachable from the spec clauses of `f` (helpers, oracle), transitively,
/// excluding `f` itself. Sorted by name.
fn reachable<'m>(module: &'m Module, f: &FnDecl) -> Vec<&'m FnDecl> {
    let mut direct: Vec<String> = Vec::new();
    for c in f.requires.iter().chain(f.ensures.iter()) {
        c.expr.walk(&mut |e| {
            if let ExprKind::Call(n, _) = &e.kind {
                direct.push(n.clone());
            }
        });
    }
    for ex in &f.examples {
        for e in ex.inputs.iter().chain(std::iter::once(&ex.output)) {
            e.walk(&mut |e| {
                if let ExprKind::Call(n, _) = &e.kind {
                    direct.push(n.clone());
                }
            });
        }
    }
    if let Some((o, _)) = &f.oracle {
        direct.push(o.clone());
    }
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut queue = direct;
    let mut out: Vec<&FnDecl> = Vec::new();
    while let Some(n) = queue.pop() {
        if n == f.name || !seen.insert(n.clone()) {
            continue;
        }
        if let Some((_, g)) = module.fns().find(|(_, g)| g.name == n) {
            queue.extend(callees(g, module));
            out.push(g);
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// User type and enum declarations mentioned by `tokens`, transitively (a variant name
/// brings in its enum). Sorted by name.
fn reachable_types<'m>(module: &'m Module, tokens: &[&String]) -> Vec<(&'m str, &'m [String])> {
    let mut decls: Vec<(&str, &[String], Vec<&str>)> = Vec::new();
    for it in &module.items {
        match &it.kind {
            ItemKind::Type(t) | ItemKind::Resource(t) => decls.push((&t.name, &t.tokens, vec![])),
            ItemKind::Enum(e) => decls.push((
                &e.name,
                &e.tokens,
                e.variants.iter().map(|v| v.name.as_str()).collect(),
            )),
            ItemKind::Fn(_) => {}
        }
    }
    let word = |t: &str| -> Option<String> {
        let rest = t.strip_prefix('T')?;
        rest.split_once(':').map(|(_, w)| w.to_string())
    };
    let mut words: BTreeSet<String> = tokens.iter().filter_map(|t| word(t)).collect();
    let mut chosen: BTreeSet<&str> = BTreeSet::new();
    loop {
        let mut changed = false;
        for (name, toks, variants) in &decls {
            let hit = words.contains(*name) || variants.iter().any(|v| words.contains(*v));
            if hit && chosen.insert(name) {
                words.extend(toks.iter().filter_map(|t| word(t)));
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let mut out: Vec<(&str, &[String])> = decls
        .iter()
        .filter(|(n, _, _)| chosen.contains(n))
        .map(|(n, t, _)| (*n, *t))
        .collect();
    out.sort_by(|a, b| a.0.cmp(b.0));
    out
}

/// SHA-256 over the signature and spec tokens (kind and length encoded) plus the full token
/// stream of every module function reachable from the spec clauses.
pub fn spec_hash(module: &Module, f: &FnDecl) -> String {
    let mut h = Sha256::new();
    h.update(f.spec_tokens.join(" ").as_bytes());
    let mut mentioned: Vec<&String> = f.spec_tokens.iter().collect();
    for g in reachable(module, f) {
        h.update(format!("\nfn {}:{} ", g.name.len(), g.name).as_bytes());
        h.update(g.full_tokens.join(" ").as_bytes());
        mentioned.extend(g.full_tokens.iter());
    }
    for (name, tokens) in reachable_types(module, &mentioned) {
        h.update(format!("\ntype {}:{} ", name.len(), name).as_bytes());
        h.update(tokens.join(" ").as_bytes());
    }
    let out = h.finalize();
    let mut s = String::from("sha256:");
    for b in out {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// `(file:module.fn, hash)` for every `pub` function, sorted by name.
pub fn entries(module: &Module, file: &str) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = module
        .fns()
        .filter(|(it, _)| it.is_pub)
        .map(|(_, f)| {
            (
                format!("{}:{}.{}", file, module.name, f.name),
                spec_hash(module, f),
            )
        })
        .collect();
    v.sort();
    v.dedup_by(|a, b| a.0 == b.0);
    v
}

fn parse_lock(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                return None;
            }
            let mut it = l.split_whitespace();
            Some((it.next()?.to_string(), it.next()?.to_string()))
        })
        .collect()
}

/// Split `file:module.fn` into (file, module, fn).
fn split_key(key: &str) -> Option<(&str, &str, &str)> {
    let (file, rest) = key.split_once(':')?;
    let (module, func) = rest.rsplit_once('.')?;
    Some((file, module, func))
}

/// New contents of `aval.lock`: other files' lines are kept, this file's are rewritten.
pub fn render(existing: Option<&str>, module: &Module, file: &str) -> String {
    let mut lines: Vec<(String, String)> = existing
        .map(parse_lock)
        .unwrap_or_default()
        .into_iter()
        .filter(|(k, _)| split_key(k).map(|(f, _, _)| f != file).unwrap_or(true))
        .collect();
    lines.extend(entries(module, file));
    lines.sort();
    let mut out = String::new();
    for (k, h) in lines {
        out.push_str(&format!("{} {}\n", k, h));
    }
    out
}

/// `E900` for changed locked specs (`reason: "changed"`) and for lock entries whose function no
/// longer exists (`reason: "missing"`, at the `module` header); `W901` for `pub` functions
/// missing from an existing lock.
pub fn check(
    module: &Module,
    lock_text: &str,
    file: &str,
    file_exists: &dyn Fn(&str) -> bool,
) -> Vec<Diag> {
    let locked = parse_lock(lock_text);
    let mut out = Vec::new();
    for (it, f) in module.fns() {
        let key = format!("{}:{}.{}", file, module.name, f.name);
        match locked.iter().find(|(k, _)| *k == key) {
            Some((_, h)) => {
                if *h != spec_hash(module, f) {
                    out.push(
                        Diag::new(
                            "E900",
                            it.pos,
                            "spec modified without human approval (aval.lock)",
                        )
                        .with_fn(&f.name)
                        .with("reason", "changed"),
                    );
                }
            }
            None => {
                if it.is_pub {
                    out.push(
                        Diag::new("W901", it.pos, "pub function is not in aval.lock")
                            .with_fn(&f.name),
                    );
                }
            }
        }
    }
    for (k, _) in &locked {
        let Some((kfile, kmodule, kfn)) = split_key(k) else {
            continue;
        };
        if kfile != file {
            // an entry of a file that is gone from the lock's directory (renamed or deleted)
            if !file_exists(kfile) {
                out.push(
                    Diag::new(
                        "E900",
                        module.header_pos,
                        "locked source file no longer exists (aval.lock)",
                    )
                    .with_fn(kfn)
                    .with("reason", "missing"),
                );
            }
            continue;
        }
        let exists =
            kmodule == module.name && module.fns().any(|(it, f)| it.is_pub && f.name == kfn);
        if !exists {
            out.push(
                Diag::new(
                    "E900",
                    module.header_pos,
                    "locked function no longer exists (aval.lock)",
                )
                .with_fn(kfn)
                .with("reason", "missing"),
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use aval_syntax::parse;

    const A: &str = "module m\npub fn b(x: Int) -> Int\n  ensures result > x\n  cost O(1)\n{\n  return x + 1\n}\npub fn a(x: Int) -> Int\n  ensures result > x\n  cost O(1)\n{\n  return x + 1\n}\nfn c(x: Int) -> Int\n{\n  return x\n}\n";

    #[test]
    fn lock_has_sorted_pub_lines_only() {
        let m = parse(A).module;
        let text = render(None, &m, "t.aval");
        let keys: Vec<&str> = text.lines().map(|l| l.split(' ').next().unwrap()).collect();
        assert_eq!(keys, vec!["t.aval:m.a", "t.aval:m.b"]);
        assert!(text
            .lines()
            .all(|l| l.contains(" sha256:") && l.len() == "t.aval:m.a sha256:".len() + 64));
    }

    #[test]
    fn body_change_is_ok_spec_change_is_e900() {
        let m = parse(A).module;
        let lock = render(None, &m, "t.aval");
        let body = parse(&A.replace("x + 1", "x + 2")).module;
        assert!(check(&body, &lock, "t.aval", &|_| true).is_empty());
        let weak = parse(&A.replace("result > x", "true")).module;
        let d = check(&weak, &lock, "t.aval", &|_| true);
        assert_eq!(d.len(), 2);
        assert!(d.iter().all(|d| d.code == "E900"));
    }

    #[test]
    fn missing_entry_is_w901_and_other_modules_are_kept() {
        let m = parse(A).module;
        let lock = render(Some("other.aval:o.f sha256:00\n"), &m, "t.aval");
        assert!(lock.contains("other.aval:o.f"));
        let more = format!(
            "{}pub fn d() -> Int\n  examples\n    => 1\n  cost O(1)\n{{\n  return 1\n}}\n",
            A
        );
        let d = check(&parse(&more).module, &lock, "t.aval", &|_| true);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].code, "W901");
    }

    const H: &str = "module m\npub fn f(xs: List<Int>) -> Int\n  ensures ok(xs, result)\n  oracle g\n  cost O(1)\n{\n  return xs.sum()\n}\nfn g(xs: List<Int>) -> Int\n{\n  return xs.sum()\n}\nfn ok(xs: List<Int>, r: Int) -> Bool\n{\n  return r == xs.sum()\n}\n";

    #[test]
    fn helper_and_oracle_bodies_are_covered() {
        let lock = render(None, &parse(H).module, "t.aval");
        let weak_helper = H.replace("return r == xs.sum()", "return true");
        let d = check(&parse(&weak_helper).module, &lock, "t.aval", &|_| true);
        assert_eq!((d.len(), d[0].code), (1, "E900"));
        let weak_oracle = H.replace(
            "fn g(xs: List<Int>) -> Int\n{\n  return xs.sum()",
            "fn g(xs: List<Int>) -> Int\n{\n  return 0",
        );
        assert_eq!(
            check(&parse(&weak_oracle).module, &lock, "t.aval", &|_| true).len(),
            1
        );
        let body = H.replace("pub fn f(xs: List<Int>) -> Int\n  ensures ok(xs, result)\n  oracle g\n  cost O(1)\n{\n  return xs.sum()", "pub fn f(xs: List<Int>) -> Int\n  ensures ok(xs, result)\n  oracle g\n  cost O(1)\n{\n  return 0");
        assert!(check(&parse(&body).module, &lock, "t.aval", &|_| true).is_empty());
    }

    #[test]
    fn stale_entries_are_reported_at_the_module_header() {
        let lock = render(None, &parse(A).module, "t.aval");
        let renamed = A.replace("pub fn a(", "pub fn z(");
        let mut d = check(&parse(&renamed).module, &lock, "t.aval", &|_| true);
        aval_syntax::diag::sort_diags(&mut d);
        let codes: Vec<&str> = d.iter().map(|d| d.code).collect();
        assert_eq!(codes, vec!["E900", "W901"]);
        assert_eq!(d[0].func.as_deref(), Some("a"));
        assert!(d[0]
            .extra
            .iter()
            .any(|(k, v)| *k == "reason" && v == "missing"));
        let deleted = "module m\npub fn b(x: Int) -> Int\n  ensures result > x\n  cost O(1)\n{\n  return x + 1\n}\n";
        assert_eq!(
            check(&parse(deleted).module, &lock, "t.aval", &|_| true).len(),
            1
        );
    }

    #[test]
    fn string_literals_cannot_mimic_tokens() {
        let a = "module m\npub fn f(x: Int) -> Bool\n  ensures result == (\"a\" == \"a K3:or\")\n  cost O(1)\n{\n  return true\n}\n";
        let b = "module m\npub fn f(x: Int) -> Bool\n  ensures result == (\"a\" == \"a\" or true)\n  cost O(1)\n{\n  return true\n}\n";
        let ma = parse(a).module;
        let mb = parse(b).module;
        assert_ne!(entries(&ma, "t.aval"), entries(&mb, "t.aval"));
    }

    #[test]
    fn module_rename_is_missing_and_types_are_hashed() {
        let lock = render(None, &parse(A).module, "t.aval");
        let renamed = A.replace("module m", "module m2");
        let mut d = check(&parse(&renamed).module, &lock, "t.aval", &|_| true);
        aval_syntax::diag::sort_diags(&mut d);
        assert_eq!(d[0].code, "E900");
        assert!(d.iter().any(|d| d.code == "W901"));
        // other files' entries are not touched
        assert!(
            check(&parse(&renamed).module, &lock, "other.aval", &|_| true)
                .iter()
                .all(|d| d.code != "E900")
        );

        let t = "module m\ntype P { v: Int }\npub fn f(p: P) -> Int\n  ensures result >= 0\n  cost O(1)\n{\n  return p.v\n}\n";
        let lock = render(None, &parse(t).module, "t.aval");
        let changed = t.replace("v: Int", "v: Byte");
        let d = check(&parse(&changed).module, &lock, "t.aval", &|_| true);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].code, "E900");
    }

    #[test]
    fn entries_of_a_vanished_file_are_missing() {
        let lock = render(None, &parse(A).module, "main.aval");
        let d = check(&parse(A).module, &lock, "main2.aval", &|f| {
            f == "main2.aval"
        });
        assert!(d
            .iter()
            .any(|d| d.code == "E900" && d.func.as_deref() == Some("a")));
        // the old file still exists: its entries belong to it, not to us
        let d = check(&parse(A).module, &lock, "main2.aval", &|_| true);
        assert!(d.iter().all(|d| d.code == "W901"));
    }
}
