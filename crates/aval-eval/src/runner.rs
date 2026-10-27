//! `aval test`: examples, property tests derived from `ensures`, oracle comparison, shrinking.

use crate::gen::{shrink_args, Gen, Rng};
use crate::interp::{Env, Interp};
use crate::value::{approx_eq, Value};
use aval_check::{ModuleInfo, Ty};
use aval_syntax::ast::*;
use aval_syntax::diag::Diag;
use serde_json::{Map, Value as Json};

pub struct Options {
    pub cases: usize,
    pub seed: u64,
    /// Step budget for one input.
    pub max_steps: u64,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            cases: 1000,
            seed: 0,
            max_steps: crate::interp::DEFAULT_STEP_LIMIT,
        }
    }
}

enum OracleSide {
    Value(Value),
    Fault(String),
}

#[derive(Default)]
struct Case {
    fault: Option<String>,
    ensures: Option<(usize, Value)>,
    oracle: Option<(Value, OracleSide)>,
}

/// Step budgets keep `aval test` fast even for exponential programs.
/// Generation and shrinking budget per function (not scaled by `--max-steps`, so a raised
/// per-input budget does not multiply the cost of property testing).
fn budget(_it: &Interp) -> u64 {
    60_000_000
}

enum Run {
    Rejected,
    Done(Case),
}

struct Best {
    weight: u64,
    args: Vec<Value>,
    detail: String,
}

#[derive(Default)]
struct Tracker {
    fault: Option<Best>,
    ens: Option<Best>,
    orc: Option<Best>,
}

impl Tracker {
    /// Keep the simplest failing input of each kind.
    fn record(&mut self, args: Vec<Value>, case: Case) {
        let w: u64 = args.iter().map(Value::weight).sum();
        let consider = |slot: &mut Option<Best>, detail: String| {
            if slot.as_ref().map(|b| w < b.weight).unwrap_or(true) {
                *slot = Some(Best {
                    weight: w,
                    args: args.clone(),
                    detail,
                });
            }
        };
        if let Some(fault) = &case.fault {
            consider(&mut self.fault, fault.clone());
        }
        if let Some((i, _)) = &case.ensures {
            consider(&mut self.ens, i.to_string());
        }
        if case.oracle.is_some() {
            consider(&mut self.orc, String::new());
        }
    }
}

fn input_obj(f: &FnDecl, args: &[Value]) -> Json {
    let mut m = Map::new();
    for (p, v) in f.params.iter().zip(args) {
        m.insert(p.name.clone(), v.to_json());
    }
    Json::Object(m)
}

fn env_of(f: &FnDecl, args: &[Value]) -> Env {
    f.params
        .iter()
        .zip(args)
        .map(|(p, v)| (p.name.clone(), v.clone()))
        .collect()
}

fn requires_hold(it: &mut Interp, f: &FnDecl, env: &Env) -> bool {
    for r in &f.requires {
        it.reset();
        match it.eval_in(&r.expr, env) {
            Ok(Value::Bool(true)) => {}
            _ => return false,
        }
    }
    true
}

fn run_budgeted(it: &mut Interp, f: &FnDecl, args: &[Value], start: u64) -> Run {
    if it.total_steps() - start > budget(it) {
        return Run::Rejected;
    }
    run_case(it, f, args)
}

fn run_case(it: &mut Interp, f: &FnDecl, args: &[Value]) -> Run {
    let env = env_of(f, args);
    if !requires_hold(it, f, &env) {
        return Run::Rejected;
    }
    it.reset();
    let got = match it.call_fn(&f.name, args.to_vec()) {
        Ok(v) => v,
        Err(fault) => {
            return Run::Done(Case {
                fault: Some(fault),
                ..Case::default()
            })
        }
    };
    let mut case = Case::default();
    let mut env2 = env.clone();
    env2.push(("result".into(), got.clone()));
    for (idx, c) in f.ensures.iter().enumerate() {
        it.reset();
        match it.eval_in(&c.expr, &env2) {
            Ok(Value::Bool(true)) => {}
            Ok(_) => {
                case.ensures = Some((idx, got.clone()));
                break;
            }
            Err(fault) => {
                return Run::Done(Case {
                    fault: Some(fault),
                    ..Case::default()
                })
            }
        }
    }
    if let Some((g, _)) = &f.oracle {
        it.reset();
        match it.call_fn(g, args.to_vec()) {
            Ok(o) => {
                if !approx_eq(&got, &o) {
                    case.oracle = Some((got, OracleSide::Value(o)));
                }
            }
            Err(fault) => case.oracle = Some((got, OracleSide::Fault(fault))),
        }
    }
    Run::Done(case)
}

pub fn run_tests(module: &Module, info: &ModuleInfo, opts: &Options) -> Vec<Diag> {
    let mut diags = Vec::new();
    let mut it = Interp::new(module, info);
    it.set_max_steps(opts.max_steps);
    let gen = Gen::new(info);
    for (idx, item) in module.items.iter().enumerate() {
        let ItemKind::Fn(f) = &item.kind else {
            continue;
        };
        let Some(sig) = info.sig_by_item.get(&idx) else {
            continue;
        };
        let tys: Vec<Ty> = sig.params.iter().map(|(_, t)| t.clone()).collect();
        test_fn(&mut it, &gen, f, &tys, opts, &mut diags);
    }
    diags
}

fn test_fn(
    it: &mut Interp,
    gen: &Gen,
    f: &FnDecl,
    tys: &[Ty],
    opts: &Options,
    out: &mut Vec<Diag>,
) {
    let (mut have100, mut have200, mut have700) = (false, false, false);

    // 1. examples
    for ex in &f.examples {
        it.reset();
        let env: Env = Vec::new();
        let inputs: Result<Vec<Value>, String> =
            ex.inputs.iter().map(|e| it.eval_in(e, &env)).collect();
        let expected = it.eval_in(&ex.output, &env);
        let (inputs, expected) = match (inputs, expected) {
            (Ok(i), Ok(e)) => (i, e),
            (i, e) => {
                let fault = i
                    .as_ref()
                    .err()
                    .or(e.as_ref().err())
                    .cloned()
                    .unwrap_or_default();
                if !have700 {
                    have700 = true;
                    let input = i
                        .ok()
                        .map(|v| input_obj(f, &v))
                        .unwrap_or_else(|| Json::Object(Map::new()));
                    out.push(
                        Diag::new(
                            "T700",
                            ex.pos,
                            format!("runtime fault in example: {}", fault),
                        )
                        .with_fn(&f.name)
                        .with("fault", fault)
                        .with("input", input),
                    );
                }
                continue;
            }
        };
        it.reset();
        match it.call_fn(&f.name, inputs.clone()) {
            Err(fault) => {
                if !have700 {
                    have700 = true;
                    out.push(
                        Diag::new("T700", f.name_pos, format!("runtime fault: {}", fault))
                            .with_fn(&f.name)
                            .with("fault", fault)
                            .with("counterexample", input_obj(f, &inputs)),
                    );
                }
            }
            Ok(got) => {
                if !approx_eq(&got, &expected) && !have100 {
                    have100 = true;
                    out.push(
                        Diag::new("T100", ex.pos, "example failed")
                            .with_fn(&f.name)
                            .with("input", input_obj(f, &inputs))
                            .with("expected", expected.to_json())
                            .with("got", got.to_json()),
                    );
                }
                let env = env_of(f, &inputs);
                if !have200 && requires_hold(it, f, &env) {
                    let mut env2 = env.clone();
                    env2.push(("result".into(), got.clone()));
                    for c in &f.ensures {
                        it.reset();
                        if let Ok(Value::Bool(false)) = it.eval_in(&c.expr, &env2) {
                            have200 = true;
                            out.push(
                                Diag::new("T200", c.expr.pos, "ensures violated")
                                    .with_fn(&f.name)
                                    .with("violated", c.text.as_str())
                                    .with("counterexample", input_obj(f, &inputs))
                                    .with("got", got.to_json()),
                            );
                            break;
                        }
                    }
                }
            }
        }
    }

    // 2. generated inputs
    if opts.cases == 0 || (f.ensures.is_empty() && f.oracle.is_none()) {
        return;
    }
    let mut rng = Rng::from_name(opts.seed, &f.name);
    let random_cases = if tys.is_empty() { 0 } else { opts.cases };
    let mut tr = Tracker::default();
    let gen_start = it.total_steps();
    let mut accepted_total = 0usize;
    for args in battery(gen, tys) {
        if let Run::Done(case) = run_case(it, f, &args) {
            accepted_total += 1;
            tr.record(args, case);
        }
    }
    let mut accepted = 0usize;
    let mut attempts = 0usize;
    let mut budget_hit = false;
    while accepted < random_cases && attempts < random_cases.saturating_mul(10) {
        if it.total_steps() - gen_start > budget(it) {
            budget_hit = true;
            break;
        }
        attempts += 1;
        let args: Vec<Value> = tys
            .iter()
            .map(|t| gen.value(t, &mut rng, 0, false))
            .collect();
        if let Run::Done(case) = run_case(it, f, &args) {
            accepted += 1;
            accepted_total += 1;
            tr.record(args, case);
        }
    }
    let vacuous = if tys.is_empty() {
        accepted_total == 0
    } else {
        opts.cases >= 10 && accepted_total * 10 < opts.cases
    };
    if vacuous && tr.fault.is_none() {
        let pos = f.requires.first().map(|c| c.expr.pos).unwrap_or(f.name_pos);
        let (reason, msg) = if budget_hit {
            (
                "budget",
                "property not exercised: the step budget stopped generation before enough inputs were tried",
            )
        } else {
            (
                "rejected",
                "property is vacuous: too few generated inputs satisfy `requires`",
            )
        };
        out.push(
            Diag::new("T210", pos, msg)
                .with_fn(&f.name)
                .with("accepted", accepted_total)
                .with("requested", if tys.is_empty() { 1 } else { opts.cases })
                .with("reason", reason),
        );
    }
    let shrink_start = it.total_steps();
    let (best_fault, best_ens, best_orc) = (tr.fault, tr.ens, tr.orc);

    if let (Some(b), false) = (best_ens, have200) {
        let clause_idx: usize = b.detail.parse().unwrap_or(0);
        let (args, cut) = shrink_args(gen, b.args, tys, &mut |a| match run_budgeted(
            it,
            f,
            a,
            shrink_start,
        ) {
            Run::Done(c) => {
                c.fault.is_none() && c.ensures.as_ref().map(|e| e.0) == Some(clause_idx)
            }
            Run::Rejected => false,
        });
        if let Run::Done(c) = run_case(it, f, &args) {
            if let Some((_, got)) = c.ensures {
                let cl = &f.ensures[clause_idx];
                let mut d = Diag::new("T200", cl.expr.pos, "ensures violated")
                    .with_fn(&f.name)
                    .with("violated", cl.text.as_str())
                    .with("counterexample", input_obj(f, &args))
                    .with("got", got.to_json());
                if cut || it.total_steps() - shrink_start > budget(it) {
                    d = d.with("shrunk", false);
                }
                out.push(d);
            }
        }
    }
    if let Some(b) = best_orc {
        let (args, cut) = shrink_args(gen, b.args, tys, &mut |a| match run_budgeted(
            it,
            f,
            a,
            shrink_start,
        ) {
            Run::Done(c) => c.fault.is_none() && c.oracle.is_some(),
            Run::Rejected => false,
        });
        if let Run::Done(c) = run_case(it, f, &args) {
            if let (Some((got, side)), Some((_, opos))) = (c.oracle, &f.oracle) {
                let mut d = Diag::new("T600", *opos, "result differs from oracle")
                    .with_fn(&f.name)
                    .with("counterexample", input_obj(f, &args))
                    .with("got", got.to_json());
                if cut || it.total_steps() - shrink_start > budget(it) {
                    d = d.with("shrunk", false);
                }
                d = match side {
                    OracleSide::Value(o) => d.with("oracle", o.to_json()),
                    OracleSide::Fault(name) => {
                        d.msg = "oracle faulted".to_string();
                        d.with("oracle", Json::Null).with("oracle_fault", name)
                    }
                };
                out.push(d);
            }
        }
    }
    if let (Some(b), false) = (best_fault, have700) {
        let fault = b.detail.clone();
        let (args, cut) = shrink_args(gen, b.args, tys, &mut |a| match run_budgeted(
            it,
            f,
            a,
            shrink_start,
        ) {
            Run::Done(c) => c.fault.as_deref() == Some(fault.as_str()),
            Run::Rejected => false,
        });
        let mut d = Diag::new("T700", f.name_pos, format!("runtime fault: {}", fault))
            .with_fn(&f.name)
            .with("fault", fault)
            .with("counterexample", input_obj(f, &args));
        if cut || it.total_steps() - shrink_start > budget(it) {
            d = d.with("shrunk", false);
        }
        out.push(d);
    }
}

/// Fixed edge-case battery: all-simplest, each parameter varied, and zipped edges.
fn battery(gen: &Gen, tys: &[Ty]) -> Vec<Vec<Value>> {
    let edges: Vec<Vec<Value>> = tys.iter().map(|t| gen.edges(t)).collect();
    if edges.iter().any(|e| e.is_empty()) {
        return Vec::new();
    }
    let mut out: Vec<Vec<Value>> = Vec::new();
    let base: Vec<Value> = edges.iter().map(|e| e[0].clone()).collect();
    out.push(base.clone());
    for i in 0..tys.len() {
        for e in edges[i].iter().skip(1) {
            let mut a = base.clone();
            a[i] = e.clone();
            out.push(a);
        }
    }
    let longest = edges.iter().map(Vec::len).max().unwrap_or(0);
    for k in 1..longest {
        out.push(edges.iter().map(|e| e[k % e.len()].clone()).collect());
    }
    let mut seen: Vec<String> = Vec::new();
    out.retain(|a| {
        let key = a
            .iter()
            .map(|v| v.to_json().to_string())
            .collect::<Vec<_>>()
            .join("|");
        if seen.contains(&key) {
            false
        } else {
            seen.push(key);
            true
        }
    });
    out
}
