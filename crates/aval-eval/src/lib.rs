//! Aval reference interpreter and self-test runner (`aval test`).
#![forbid(unsafe_code)]

mod gen;
mod interp;
mod runner;
mod value;

pub use gen::Rng;
pub use interp::{interp_cmp, Interp};
pub use runner::{run_tests, Options};
pub use value::{approx, approx_eq, Value};

#[cfg(test)]
mod tests {
    use super::*;
    use aval_check::check_source;

    fn run(src: &str, cases: usize) -> Vec<serde_json::Value> {
        let src = src.to_string();
        std::thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn(move || {
                let c = check_source(&src);
                assert!(!c.has_errors(), "{:?}", c.diags);
                run_tests(
                    &c.module,
                    c.info.as_ref().unwrap(),
                    &Options {
                        cases,
                        seed: 0,
                        ..Options::default()
                    },
                )
                .iter()
                .map(|d| d.to_json())
                .collect()
            })
            .unwrap()
            .join()
            .unwrap()
    }

    fn mean(body: &str) -> String {
        format!("module m\nfn mean(xs: List<Float>) -> Float\n  requires xs.len() > 0\n  ensures approx(result * xs.len().float(), xs.sum())\n{{\n  return {}\n}}\n", body)
    }

    fn range_fn(body: &str, ex: &str) -> Vec<serde_json::Value> {
        run(
            &format!(
                "module m\nfn f(n: Int) -> Int\n  examples\n    {}\n{{\n  return {}\n}}\n",
                ex, body
            ),
            0,
        )
    }

    #[test]
    fn range_methods_work() {
        assert!(range_fn("(0..n).fold(0, |a, i| a + i)", "5 => 10").is_empty());
        assert!(range_fn("(1..=n).sum() + (1..=n).product()", "4 => 34").is_empty());
        assert!(range_fn("(5..2).sum() + (5..2).product() + (5..2).len()", "0 => 1").is_empty());
        assert!(range_fn("(1..=n).map(|i| i * 2).sum()", "3 => 12").is_empty());
        assert!(range_fn("(1..=n).reverse().take(2).sum()", "5 => 9").is_empty());
        assert!(range_fn("(0..n).skip(3).len()", "5 => 2").is_empty());
    }

    #[test]
    fn range_min_of_empty_faults_and_huge_sum_hits_step_limit() {
        let d = range_fn("(3..n).min()", "3 => 0");
        assert_eq!(d[0]["fault"], "empty_list");
        let d = range_fn("(0..n).sum()", "1000000000 => 0");
        assert_eq!(d[0]["fault"], "step_limit");
        let d = range_fn("(0..n).reverse().len()", "1000000000 => 0");
        assert_eq!(d[0]["fault"], "memory_limit");
    }

    #[test]
    fn range_of_2_pow_64_elements_faults_memory_limit() {
        for m in [
            "skip(0)",
            "reverse()",
            "take(9223372036854775807)",
            "map(|i| i)",
        ] {
            let d = run(
                &format!("module m\nfn f(a: Int) -> Int\n  examples\n    0 => 0\n{{\n  return ((a - 9223372036854775807 - 1)..=(a + 9223372036854775807)).{}.len()\n}}\n", m),
                0,
            );
            assert_eq!(d[0]["fault"], "memory_limit", "{m}");
        }
    }

    #[test]
    fn huge_range_reductions_end_with_step_limit() {
        for body in [
            "(0..n).min()",
            "(0..n).max()",
            "(0..n).count(5)",
            "(0..n).product()",
            "(0..n).fold(0, |a, i| a + i)",
            "(if (0..n).any(|i| i < 0) { 1 } else { 0 })",
            "(if (0..n).all(|i| i >= 0) { 1 } else { 0 })",
        ] {
            let d = range_fn(body, "1000000000 => 0");
            assert_eq!(d[0]["fault"], "step_limit", "{body}");
        }
        let d = range_fn("(0..n).map(|i| i).len()", "1000000000 => 0");
        assert_eq!(d[0]["fault"], "memory_limit");
    }

    #[test]
    fn range_len_overflow_faults() {
        let d = run(
            "module m\nfn f(a: Int) -> Int\n  examples\n    0 => 0\n{\n  return (a - 9223372036854775807 - 1..a + 9223372036854775807).len()\n}\n",
            0,
        );
        assert_eq!(d[0]["fault"], "overflow");
    }

    #[test]
    fn correct_mean_passes_and_buggy_mean_is_shrunk() {
        assert!(run(&mean("xs.sum() / xs.len().float()"), 1000).is_empty());
        let d = run(&mean("xs.sum() / 2.0"), 1000);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0]["code"], "T200");
        assert_eq!(
            d[0]["violated"],
            "approx(result * xs.len().float(), xs.sum())"
        );
        assert_eq!(d[0]["counterexample"]["xs"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn int_and_list_counterexamples_are_minimal() {
        let d = run(
            "module m\nfn f(x: Int) -> Int\n  ensures result < 100\n{\n  return x\n}\n",
            1000,
        );
        assert_eq!(d[0]["counterexample"], serde_json::json!({"x": 100}));
        let d = run("module m\nfn f(xs: List<Int>) -> Int\n  ensures result != 7\n{\n  return xs.sum()\n}\n", 1000);
        assert_eq!(d[0]["counterexample"], serde_json::json!({"xs": [7]}));
    }

    #[test]
    fn runtime_faults() {
        let d = run(
            "module m\nfn inv(x: Int) -> Int\n  ensures true\n{\n  return 100 / x\n}\n",
            100,
        );
        assert_eq!(
            (d[0]["code"].as_str(), d[0]["fault"].as_str()),
            (Some("T700"), Some("division_by_zero"))
        );
        assert_eq!(d[0]["counterexample"], serde_json::json!({"x": 0}));
        let d = run(
            "module m\nfn sq(x: Int) -> Int\n  ensures result >= 0\n{\n  return x * x\n}\n",
            100,
        );
        assert_eq!(d[0]["fault"], "overflow");
        let d = run(
            "module m\nfn f(xs: List<Int>) -> Int\n  ensures true\n{\n  return xs.min()\n}\n",
            100,
        );
        assert_eq!(d[0]["fault"], "empty_list");
    }

    #[test]
    fn requires_filters_inputs() {
        let src = "module m\nfn inv(x: Int) -> Int\n  requires x != 0\n  ensures result == 100 / x\n{\n  return 100 / x\n}\n";
        assert!(run(src, 1000).is_empty());
    }

    #[test]
    fn examples_and_oracle() {
        let d = run(
            "module m\nfn double(x: Int) -> Int\n  examples\n    3 => 6\n{\n  return x + 2\n}\n",
            10,
        );
        assert_eq!(d.len(), 1);
        assert_eq!(
            (
                d[0]["code"].as_str(),
                d[0]["expected"].as_i64(),
                d[0]["got"].as_i64()
            ),
            (Some("T100"), Some(6), Some(5))
        );
        assert_eq!(d[0]["input"], serde_json::json!({"x": 3}));
        let d = run("module m\nfn f(xs: List<Int>) -> Int\n  oracle g\n{\n  return xs.len()\n}\nfn g(xs: List<Int>) -> Int\n{\n  return xs.sum()\n}\n", 100);
        assert_eq!(d[0]["code"], "T600");
        let d = run("module m\nfn f(x: Int) -> Int\n  ensures result > 10\n  examples\n    5 => 5\n{\n  return x\n}\n", 0);
        assert_eq!(
            (d[0]["code"].as_str(), d[0]["counterexample"].clone()),
            (Some("T200"), serde_json::json!({"x": 5}))
        );
    }

    #[test]
    fn zero_cases_still_runs_examples() {
        let d = run(
            "module m\nfn f(x: Int) -> Int\n  examples\n    1 => 2\n{\n  return x\n}\n",
            0,
        );
        assert_eq!(d[0]["code"], "T100");
    }

    #[test]
    fn runs_are_deterministic() {
        let src = mean("xs.sum() / 2.0");
        assert_eq!(run(&src, 1000), run(&src, 1000));
    }

    #[test]
    fn language_semantics() {
        let ok = |src: &str| assert!(run(src, 10).is_empty(), "{}", src);
        ok("module m\nfn f() -> Int\n  examples\n    => 7\n{\n  return 1 + 2 * 3\n}\n");
        ok("module m\nfn f(xs: List<Int>) -> List<Int>\n  examples\n    [1, 2] => [2, 4]\n{\n  return for x in xs { x * 2 }\n}\n");
        ok("module m\ntype Point { x: Int, y: Int }\nfn f() -> Int\n  examples\n    => 2\n{\n  return Point { x: 1, y: 2 }.y\n}\n");
        ok("module m\nfn half(x: Int) -> Result<Int, String>\n{\n  return if x % 2 == 0 { Ok(x / 2) } else { Err(\"odd\") }\n}\nfn q(x: Int) -> Result<Int, String>\n  examples\n    8 => Ok(2)\n    6 => Err(\"odd\")\n{\n  return Ok(half(half(x)?)?)\n}\n");
        ok("module m\nenum Shape { Circle(Float), Square(Float) }\nfn area(s: Shape) -> Float\n  examples\n    Square(2.0) => 4.0\n{\n  return match s {\n    Circle(r) => 3.0 * r * r\n    Square(c) => c * c\n  }\n}\n");
        ok("module m\nfn f(p: Byte) -> Byte\n  examples\n    200 => 255\n    3 => 0\n{\n  return if p > 128 { 255 } else { 0 }\n}\n");
        ok("module m\nfn f(xs: List<Int>) -> Int\n  examples\n    [1, 2, 3] => 6\n{\n  return xs.fold(0, |acc, x| acc + x)\n}\n");
        ok("module m\nfn f() -> Float\n  examples\n    => 0.3333333333333333\n{\n  return 1.0 / 3.0\n}\n");
        ok("module m\nfn f(x: Int) -> Int\n  examples\n    -7 => -1\n{\n  return x % 3\n}\n");
    }

    #[test]
    fn recursive_types_do_not_crash_generation() {
        let ok = "module m\ntype T { l: L }\nenum L { Cons(T), Nil }\nfn f(t: T) -> Bool\n  ensures result\n{\n  return true\n}\n";
        assert!(run(ok, 200).is_empty());
        let tree = "module m\nenum Tree { Leaf(Int), Node(List<Tree>) }\nfn f(t: Tree) -> Bool\n  ensures result\n{\n  return true\n}\n";
        assert!(run(tree, 200).is_empty());
    }

    #[test]
    fn extremes_reach_list_elements_and_struct_fields() {
        let d = run("module m\nfn f(xs: List<Int>) -> List<Int>\n  ensures result.len() == xs.len()\n{\n  return xs.map(|x| x.abs())\n}\n", 1000);
        assert_eq!(
            (d[0]["code"].as_str(), d[0]["fault"].as_str()),
            (Some("T700"), Some("overflow"))
        );
        let d = run("module m\ntype P { x: Int, y: Int }\nfn f(p: P) -> Int\n  ensures result > p.x\n{\n  return p.x + 1\n}\n", 1000);
        assert_eq!(d[0]["fault"], "overflow");
    }

    #[test]
    fn vacuous_properties_are_reported() {
        let d = run("module m\nfn f(x: Int) -> Int\n  requires x == 4242\n  ensures result == x\n{\n  return x\n}\n", 1000);
        assert_eq!(d[0]["code"], "T210");
        assert_eq!(d[0]["requested"], 1000);
        assert_eq!(d[0]["reason"], "rejected");
        let ok = run("module m\nfn f(xs: List<Int>) -> Int\n  requires xs.len() > 8\n  ensures result == xs.len()\n{\n  return xs.len()\n}\n", 1000);
        assert!(ok.is_empty(), "{:?}", ok);
        // fewer than 10 requested cases never counts as vacuous
        let few = run("module m\nfn f(x: Int) -> Int\n  requires x == 4242\n  ensures result == x\n{\n  return x\n}\n", 5);
        assert!(few.is_empty());
    }

    #[test]
    fn faulting_examples_are_reported() {
        let d = run("module m\nfn f(xs: List<Int>) -> Int\n  examples\n    [1] => [].min()\n{\n  return 1\n}\n", 0);
        assert_eq!(
            (d[0]["code"].as_str(), d[0]["fault"].as_str()),
            (Some("T700"), Some("empty_list"))
        );
    }

    #[test]
    fn oracle_faults_are_reported_as_t600() {
        let src = "module m\nfn f(x: Int) -> Int\n  oracle g\n{\n  return x\n}\nfn g(x: Int) -> Int\n{\n  return 100 / x\n}\n";
        let d = run(src, 50);
        assert_eq!(d[0]["code"], "T600");
        assert_eq!(d[0]["oracle_fault"], "division_by_zero");
        assert_eq!(d[0]["oracle"], serde_json::Value::Null);
    }

    #[test]
    fn exponential_recursion_hits_the_step_budget() {
        let src = "module m\nfn fib(n: Int) -> Int\n  requires n >= 0\n  ensures result >= 0\n{\n  return if n < 2 { n } else { fib(n - 1) + fib(n - 2) }\n}\n";
        let d = run(src, 20);
        assert_eq!(
            (d[0]["code"].as_str(), d[0]["fault"].as_str()),
            (Some("T700"), Some("step_limit"))
        );
    }

    #[test]
    fn list_counterexample_keeps_minimal_length() {
        let d = run("module m\nfn f(xs: List<Int>) -> Int\n  ensures result != 7\n{\n  return xs.len()\n}\n", 1000);
        assert_eq!(
            d[0]["counterexample"],
            serde_json::json!({"xs": [0, 0, 0, 0, 0, 0, 0]})
        );
    }

    #[test]
    fn recursive_enum_counterexample_is_small() {
        let src = "module m\nenum L { Cons(Int, L), Nil }\nfn len(l: L) -> Int\n  ensures result < 3\n{\n  return match l {\n    Cons(_, r) => 1 + len(r)\n    Nil => 0\n  }\n}\n";
        let d = run(src, 1000);
        assert_eq!(
            d[0]["counterexample"],
            serde_json::json!({"l": {"Cons": [0, {"Cons": [0, {"Cons": [0, "Nil"]}]}]}})
        );
        assert_eq!(d[0]["got"], 3);
    }

    #[test]
    fn vacuous_without_parameters_and_with_costly_requires() {
        let d = run(
            "module m\nfn f() -> Int\n  requires false\n  ensures result == 1\n{\n  return 2\n}\n",
            1000,
        );
        assert_eq!(
            (
                d[0]["code"].as_str(),
                d[0]["accepted"].as_i64(),
                d[0]["requested"].as_i64()
            ),
            (Some("T210"), Some(0), Some(1))
        );
        let src = "module m\nfn slow(x: Int) -> Bool\n{\n  return (for i in 0..1000000 { i }).len() > 0\n}\nfn f(x: Int) -> Int\n  requires slow(x) and x == 4242\n  ensures result == x\n{\n  return x + 1\n}\n";
        let d = run(src, 1000);
        assert_eq!(d[0]["code"], "T210");
        assert_eq!(d[0]["reason"], "budget");
    }
}
