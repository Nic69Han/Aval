//! Aval checker: name resolution, bidirectional typing, effects, spec rules and spec lock.
#![forbid(unsafe_code)]

pub mod exh;
pub mod info;
pub mod lock;
pub mod prelude;
pub mod ty;
pub mod typeck;

pub use info::{FnSig, ModuleInfo};
pub use ty::Ty;

use aval_syntax::ast::{ItemKind, Module};
use aval_syntax::diag::{has_errors, sort_diags, Diag};

pub struct Checked {
    pub module: Module,
    /// `None` after a lexical error (nothing else is checked).
    pub info: Option<ModuleInfo>,
    /// A lexical or syntax error was found (items that parsed are still checked).
    pub syntax_error: bool,
    /// Sorted diagnostics of stages 1-4 (lock diagnostics are added by `lock::check`).
    pub diags: Vec<Diag>,
}

/// Run lexing, parsing, resolution, typing, effects and spec rules (spec section 12, stages 1-4).
pub fn check_source(src: &str) -> Checked {
    let parsed = aval_syntax::parse(src);
    let module = parsed.module;
    let syntax_error = !parsed.diags.is_empty();
    let lexical = parsed.diags.iter().any(|d| d.code == "E001");
    if lexical {
        let mut diags = parsed.diags;
        sort_diags(&mut diags);
        return Checked {
            module,
            info: None,
            syntax_error,
            diags,
        };
    }
    let mut diags = parsed.diags;
    let mut info = info::build(&module, &mut diags);
    for (idx, it) in module.items.iter().enumerate() {
        if let ItemKind::Fn(f) = &it.kind {
            let Some(sig) = info.sig_by_item.get(&idx).cloned() else {
                continue;
            };
            let mut fc = typeck::Fc::new(src, &info, f, sig.ret.clone());
            fc.check_fn(it, f, &sig.params);
            diags.append(&mut fc.diags);
            let (b, s) = (fc.byte_lits, fc.float_sums);
            info.byte_lits.extend(b);
            info.float_sums.extend(s);
        }
    }
    sort_diags(&mut diags);
    Checked {
        module,
        info: Some(info),
        syntax_error,
        diags,
    }
}

impl Checked {
    pub fn has_errors(&self) -> bool {
        has_errors(&self.diags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(src: &str) -> Vec<String> {
        check_source(src)
            .diags
            .iter()
            .map(|d| d.code.to_string())
            .collect()
    }

    fn first(src: &str) -> Diag {
        check_source(src)
            .diags
            .into_iter()
            .next()
            .expect("a diagnostic")
    }

    fn field(d: &Diag, key: &str) -> serde_json::Value {
        d.extra
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    }

    fn func(body: &str) -> String {
        format!(
            "module m\nfn f(x: Int, xs: List<Float>) -> Int\n{{\n  return {}\n}}\n",
            body
        )
    }

    #[test]
    fn clean_module_has_no_diagnostics() {
        let src = "module m\npub fn mean(xs: List<Float>) -> Float\n  requires xs.len() > 0\n  ensures approx(result * xs.len().float(), xs.sum())\n  examples\n    [1.0, 2.0, 3.0] => 2.0\n  cost O(xs.len())\n{\n  return xs.sum() / xs.len().float()\n}\n";
        assert!(codes(src).is_empty(), "{:?}", check_source(src).diags);
    }

    #[test]
    fn type_mismatches() {
        let d = first("module m\nfn f() -> Int\n{\n  return 1.0\n}\n");
        assert_eq!(
            (d.code, field(&d, "expected"), field(&d, "found")),
            ("E301", "Int".into(), "Float".into())
        );
        assert_eq!(codes(&func("if 1 { 2 } else { 3 }")), vec!["E301"]);
        assert_eq!(codes(&func("xs.sum() / xs.len()")), vec!["E301"]);
        assert_eq!(codes(&func("[1, 2.0].len()")), vec!["E301"]);
    }

    #[test]
    fn byte_literals_follow_the_expected_type() {
        let ok = "module m\nfn f(p: Byte) -> Byte\n{\n  return if p > 128 { 255 } else { 0 }\n}\n";
        assert!(codes(ok).is_empty());
        assert_eq!(
            codes("module m\nfn f() -> Byte\n{\n  return 256\n}\n"),
            vec!["E302"]
        );
    }

    #[test]
    fn unknown_member_lists_sorted_alternatives() {
        let d = first(&func("xs.average()"));
        assert_eq!(d.code, "E204");
        assert_eq!(field(&d, "found"), "xs.average");
        assert_eq!(field(&d, "on"), "List<Float>");
        let avail = field(&d, "avail");
        let names: Vec<&str> = avail
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
        assert!(names.contains(&"sum"));
    }

    #[test]
    fn names_and_duplicates() {
        let d = first(&func("y"));
        assert_eq!((d.code, field(&d, "name")), ("E101", "y".into()));
        assert_eq!(
            codes("module m\nfn f() -> Int\n{\n  return 1\n}\nfn f() -> Int\n{\n  return 2\n}\n"),
            vec!["E102"]
        );
        assert_eq!(
            codes("module m\nfn f(x: Int) -> Int\n{\n  let x: Int = 1\n  return x\n}\n"),
            vec!["E102"]
        );
        let d = first("module m\nimport vision.camera { Camera }\n");
        assert_eq!(
            (d.code, field(&d, "module")),
            ("E103", "vision.camera".into())
        );
        assert_eq!(codes("module m\ntype O { v: Int }\n"), vec!["E105"]);
    }

    #[test]
    fn call_before_declaration_and_block_scope() {
        assert!(codes(
            "module m\nfn a() -> Int\n{\n  return b()\n}\nfn b() -> Int\n{\n  return 1\n}\n"
        )
        .is_empty());
        let src = "module m\nfn f(x: Int) -> Int\n{\n  return if x > 0 {\n    let y: Int = 1\n    y\n  } else { y }\n}\n";
        assert_eq!(codes(src), vec!["E101"]);
    }

    #[test]
    fn arity_and_question_mark() {
        assert_eq!(codes("module m\nfn g(x: Int) -> Int\n{\n  return x\n}\nfn f() -> Int\n{\n  return g(1, 2)\n}\n"), vec!["E303"]);
        let src = "module m\nfn half(x: Int) -> Result<Int, String>\n{\n  return Ok(x)\n}\nfn f(x: Int) -> Int\n{\n  return half(x)?\n}\n";
        assert_eq!(codes(src), vec!["E301"]);
    }

    #[test]
    fn match_exhaustiveness() {
        let src = "module m\nenum Color { Red, Green, Blue }\nfn f(c: Color) -> Int\n{\n  return match c {\n    Red => 1\n    Green => 2\n  }\n}\n";
        let d = first(src);
        assert_eq!(d.code, "E310");
        assert_eq!(field(&d, "missing"), serde_json::json!(["Blue"]));
    }

    #[test]
    fn effects_and_par() {
        let e = |extra: &str, body: &str| {
            format!(
                "module m\nfn f(xs: List<Int>) -> Bool\n{}{{\n  return {}\n}}\n",
                extra, body
            )
        };
        let d = first(&e("", "print(\"hi\")"));
        assert_eq!((d.code, field(&d, "effect")), ("E420", "io.stdout".into()));
        assert!(codes(&e("  effects io.stdout\n", "print(\"hi\")")).is_empty());
        assert_eq!(codes(&e("  effects none\n", "print(\"x\")")), vec!["E420"]);
        assert_eq!(
            codes(&e(
                "  effects io.stdout\n",
                "par for x in xs { print(\"x\") }.len() > 0"
            )),
            vec!["E421"]
        );
        assert_eq!(
            codes(&e("  ensures print(\"x\")\n  effects io.stdout\n", "true")),
            vec!["E420"]
        );
    }

    #[test]
    fn spec_rules() {
        assert_eq!(
            codes("module m\npub fn f(x: Int) -> Int\n  cost O(1)\n{\n  return x\n}\n"),
            vec!["E510"]
        );
        assert_eq!(
            codes("module m\npub fn f(x: Int) -> Int\n  ensures result == x\n{\n  return x\n}\n"),
            vec!["E511"]
        );
        assert!(codes("module m\nfn f(x: Int) -> Int\n{\n  return x\n}\n").is_empty());
        assert_eq!(
            codes("module m\nfn f(x: Int) -> Int\n  requires result > 0\n{\n  return x\n}\n"),
            vec!["E106"]
        );
        assert_eq!(codes(&func("result")), vec!["E106"]);
        assert_eq!(
            codes("module m\nfn f(x: Int) -> Int\n  ensures result + 1\n{\n  return x\n}\n"),
            vec!["E301"]
        );
        assert_eq!(
            codes(
                "module m\nfn f(x: Int, y: Int) -> Int\n  examples\n    1 => 2\n{\n  return x\n}\n"
            ),
            vec!["E303"]
        );
    }

    #[test]
    fn oracle_signature_and_float_equality() {
        let src = "module m\nfn f(x: Int) -> Int\n  oracle g\n{\n  return x\n}\nfn g(x: Float) -> Int\n{\n  return 1\n}\n";
        assert_eq!(codes(src), vec!["E301"]);
        let w = check_source(
            "module m\nfn f() -> Float\n  ensures result == 1.0\n{\n  return 1.0\n}\n",
        );
        assert_eq!(w.diags.len(), 1);
        assert_eq!(w.diags[0].code, "W210");
        assert!(!w.has_errors());
    }

    #[test]
    fn diagnostics_are_ordered_and_recovered() {
        let src = "module m\nfn f(x: Int) -> Int\n{\n  let a: Int = 1.0\n  return y\n}\nfn g() -> Int\n{\n  return z\n}\n";
        assert_eq!(codes(src), vec!["E301", "E101", "E101"]);
    }

    #[test]
    fn broken_items_do_not_cascade() {
        let src =
            "module m\nfn f(x: Int) Int\n{\n  return x\n}\nfn g() -> Int\n{\n  return f(1)\n}\n";
        assert_eq!(codes(src), vec!["E002"]);
    }

    #[test]
    fn lexical_error_stops_everything() {
        let c = check_source("module m\nfn f() -> Int\n{\n  return 1 @ 2\n}\n");
        assert_eq!(c.diags.len(), 1);
        assert_eq!(c.diags[0].code, "E001");
    }

    #[test]
    fn lambdas_infer_parameter_types() {
        let src = "module m\nfn f(xs: List<Int>) -> List<Bool>\n{\n  return xs.map(|x| x > 0)\n}\n";
        assert!(codes(src).is_empty());
        let src =
            "module m\nfn f(xs: List<Int>) -> Int\n{\n  return xs.fold(0, |acc, x| acc + x)\n}\n";
        assert!(codes(src).is_empty());
    }

    #[test]
    fn empty_list_literals_are_inferred_not_any() {
        let ret = |ty: &str, body: &str| {
            format!(
                "module m\nfn f(x: Int, xs: List<Int>) -> {}\n{{\n  return {}\n}}\n",
                ty, body
            )
        };
        assert_eq!(codes(&ret("List<String>", "[].push(x)")), vec!["E301"]);
        assert!(codes(&ret("List<Int>", "[].push(x)")).is_empty());
        assert_eq!(
            codes(&ret("List<String>", "xs.fold([], |acc, y| acc.push(y))")),
            vec!["E301"]
        );
        assert!(codes(&ret("List<Int>", "xs.fold([], |acc, y| acc.push(y))")).is_empty());
        assert_eq!(
            codes(&ret("Int", "[].push(1).push(\"a\").sum()")),
            vec!["E301"]
        );
        assert_eq!(codes(&ret("String", "[].sum()")), vec!["E301"]);
        assert!(codes(&ret("Int", "[].min()")).is_empty());
    }

    #[test]
    fn field_called_as_method_is_e204() {
        let src = "module m\ntype Point { x: Int, y: Int }\nfn f(p: Point) -> Int\n{\n  return p.x()\n}\n";
        let d = first(src);
        assert_eq!((d.code, field(&d, "found")), ("E204", "p.x".into()));
    }

    #[test]
    fn uninhabited_types_are_rejected() {
        assert_eq!(codes("module m\ntype N { n: N }\n"), vec!["E312"]);
        assert!(codes("module m\ntype T { l: L }\nenum L { Cons(T), Nil }\n").is_empty());
        assert_eq!(codes("module m\nenum E { A(E), B(E) }\n"), vec!["E312"]);
    }

    #[test]
    fn examples_are_closed() {
        let d = first("module m\nfn f(x: Int) -> Int\n  examples\n    x => 99\n{\n  return x\n}\n");
        assert_eq!((d.code, field(&d, "name")), ("E101", "x".into()));
        let src = "module m\nfn g(x: Int) -> Int\n{\n  return x\n}\nfn f(x: Int) -> Int\n  examples\n    1 => g(1)\n{\n  return x\n}\n";
        assert_eq!(codes(src), vec!["E101"]);
    }

    #[test]
    fn oracles_must_be_pure_and_distinct() {
        assert_eq!(
            codes("module m\nfn f(x: Int) -> Int\n  oracle f\n{\n  return x\n}\n"),
            vec!["E107"]
        );
        let src = "module m\nfn f(x: Int) -> Int\n  oracle g\n{\n  return x\n}\nfn g(x: Int) -> Int\n  effects io.stdout\n{\n  return x\n}\n";
        assert_eq!(codes(src), vec!["E107"]);
    }

    #[test]
    fn space_before_generic_arguments() {
        let src = "module m\nfn f(xs: List <Int>) -> List <Int>\n  ensures result.len() >= 0\n{\n  return xs\n}\n";
        assert!(codes(src).is_empty(), "{:?}", check_source(src).diags);
    }

    #[test]
    fn empty_list_inference_through_branches_and_nesting() {
        let ret = |ty: &str, body: &str| {
            format!(
                "module m\nfn f(x: Int, b: Bool, xs: List<Int>) -> {}\n{{\n  return {}\n}}\n",
                ty, body
            )
        };
        assert_eq!(
            codes(&ret("List<String>", "xs.fold([], |acc, y| [y])")),
            vec!["E301"]
        );
        assert_eq!(
            codes(&ret(
                "List<String>",
                "(if x > 0 { [] } else { [x] }).reverse()"
            )),
            vec!["E301"]
        );
        assert_eq!(
            codes(&ret(
                "List<String>",
                "(match b {\n    true => []\n    false => [1]\n  }).sort()"
            )),
            vec!["E301"]
        );
        assert_eq!(
            codes(&ret("String", "(if x > 0 { [] } else { [x] }).max()")),
            vec!["E301"]
        );
        assert_eq!(
            codes(&ret("List<List<String>>", "[[]].push([x])")),
            vec!["E301"]
        );
        assert!(codes(&ret(
            "List<Int>",
            "(if x > 0 { [] } else { [x] }).reverse()"
        ))
        .is_empty());
    }

    #[test]
    fn uninferable_empty_list_is_e313_once() {
        let f = |ty: &str, body: &str| {
            format!("module m\nfn f() -> {}\n{{\n  return {}\n}}\n", ty, body)
        };
        assert_eq!(codes(&f("Bool", "[].sum() == [].sum()")), vec!["E313"]);
        assert!(codes(&f("Int", "[].sum()")).is_empty());
    }

    #[test]
    fn e204_found_keeps_the_parenthesis() {
        let d = first(&func("(xs).average()"));
        assert_eq!(field(&d, "found"), "(xs).average");
    }

    #[test]
    fn return_type_with_space_before_generic_args() {
        let src = "module m\nfn f(xs: List <Int>) -> List <Int>\n  ensures result.len() >= 0\n{\n  return xs\n}\n";
        assert!(codes(src).is_empty());
    }

    #[test]
    fn fold_accumulator_reaches_a_fixpoint() {
        let f = |ret: &str, body: &str, extra: &str| {
            format!(
                "module m\n{}fn f(xs: List<Int>) -> {}\n{{\n  return {}\n}}\n",
                extra, ret, body
            )
        };
        let g = "fn g(s: List<String>) -> List<String>\n{\n  return s\n}\n";
        assert_eq!(
            codes(&f(
                "List<Int>",
                "xs.fold([], |acc, x| acc.map(|y| y.len()).push(x))",
                ""
            )),
            vec!["E204"]
        );
        assert_eq!(
            codes(&f("List<Int>", "xs.fold([], |acc, x| g(acc).push(x))", g)),
            vec!["E301"]
        );
        let r = "xs.fold(Err(0), |acc, x| if acc.is_ok() {\n    let s: String = acc.unwrap()\n    Ok(s.len())\n  } else { Ok(x) }).unwrap()";
        assert_eq!(codes(&f("Int", r, "")), vec!["E301"]);
        // valid shapes stay valid and report nothing twice
        assert!(codes(&f(
            "List<Int>",
            "xs.fold([], |acc, x| acc.push(x)).sort()",
            ""
        ))
        .is_empty());
        assert!(codes(&f("Int", "xs.fold([], |acc, x| [x]).sum()", "")).is_empty());
    }

    #[test]
    fn max_of_empty_list_into_unordered_type() {
        let src = "module m\nfn f() -> Bool\n{\n  let b: Bool = [].max()\n  return b\n}\n";
        assert_eq!(codes(src), vec!["E204"]);
    }
}
