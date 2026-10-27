//! Tree-walking evaluator over the checked AST.

use crate::value::{approx, Value};
use aval_check::ModuleInfo;
use aval_syntax::ast::*;
use aval_syntax::diag::Pos;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::rc::Rc;

pub type Env = Vec<(String, Value)>;

pub enum Flow {
    Fault(String),
    /// Early exit of the enclosing function through `?` carrying the `Err` value.
    Early(Value),
}

type R = Result<Value, Flow>;

fn fault<T>(name: &str) -> Result<T, Flow> {
    Err(Flow::Fault(name.to_string()))
}

/// Default evaluation steps allowed for one input (design: 10,000,000).
pub const DEFAULT_STEP_LIMIT: u64 = 10_000_000;
/// Largest `String` (bytes) or `List` (elements) a value may have.
const MEMORY_LIMIT: usize = 10_000_000;
/// Nested user calls allowed; deeper is the fault `stack_limit`.
const DEPTH_LIMIT: u32 = 10_000;

type LambdaDef<'a> = (&'a [(String, Pos)], &'a Expr);

pub struct Interp<'a> {
    info: &'a ModuleInfo,
    fns: BTreeMap<&'a str, &'a FnDecl>,
    lambdas: BTreeMap<u32, LambdaDef<'a>>,
    steps: u64,
    max_steps: u64,
    total: u64,
    depth: u32,
}

fn cmp_values(a: &Value, b: &Value) -> Option<Ordering> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Some(x.cmp(y)),
        (Value::Byte(x), Value::Byte(y)) => Some(x.cmp(y)),
        (Value::Float(x), Value::Float(y)) => x.partial_cmp(y),
        (Value::Str(x), Value::Str(y)) => Some(x.cmp(y)),
        _ => None,
    }
}

fn total_cmp(a: &Value, b: &Value) -> Ordering {
    match (a, b) {
        (Value::Float(x), Value::Float(y)) => x.total_cmp(y),
        _ => cmp_values(a, b).unwrap_or(Ordering::Equal),
    }
}

impl<'a> Interp<'a> {
    pub fn new(module: &'a Module, info: &'a ModuleInfo) -> Interp<'a> {
        let mut fns = BTreeMap::new();
        let mut lambdas = BTreeMap::new();
        for (_, f) in module.fns() {
            fns.entry(f.name.as_str()).or_insert(f);
            f.walk_all(&mut |e| {
                if let ExprKind::Lambda(ps, body) = &e.kind {
                    lambdas.insert(e.id, (ps.as_slice(), &**body));
                }
            });
        }
        Interp {
            info,
            fns,
            lambdas,
            steps: 0,
            max_steps: DEFAULT_STEP_LIMIT,
            total: 0,
            depth: 0,
        }
    }

    /// Step budget for one input (`aval test --max-steps`).
    pub fn set_max_steps(&mut self, n: u64) {
        self.max_steps = n;
    }

    pub fn max_steps(&self) -> u64 {
        self.max_steps
    }

    /// Steps evaluated since creation (used for per-function budgets).
    pub fn total_steps(&self) -> u64 {
        self.total
    }

    pub fn reset(&mut self) {
        self.steps = 0;
        self.depth = 0;
    }

    /// Call a user function by name; `Err` carries the fault name.
    pub fn call_fn(&mut self, name: &str, args: Vec<Value>) -> Result<Value, String> {
        match self.call_user(name, args) {
            Ok(v) | Err(Flow::Early(v)) => Ok(v),
            Err(Flow::Fault(f)) => Err(f),
        }
    }

    /// Evaluate an expression in an environment.
    pub fn eval_in(&mut self, e: &Expr, env: &Env) -> Result<Value, String> {
        let mut env = env.clone();
        match self.eval(e, &mut env) {
            Ok(v) | Err(Flow::Early(v)) => Ok(v),
            Err(Flow::Fault(f)) => Err(f),
        }
    }

    fn call_user(&mut self, name: &str, args: Vec<Value>) -> R {
        let Some(f) = self.fns.get(name).copied() else {
            return fault("unknown_function");
        };
        if self.depth >= DEPTH_LIMIT {
            return fault("stack_limit");
        }
        self.depth += 1;
        let mut env: Env = f
            .params
            .iter()
            .zip(args)
            .map(|(p, v)| (p.name.clone(), v))
            .collect();
        let r = self.body(&f.body, &mut env);
        self.depth -= 1;
        match r {
            Err(Flow::Early(v)) => Ok(v),
            other => other,
        }
    }

    fn body(&mut self, b: &Body, env: &mut Env) -> R {
        let mark = env.len();
        for l in &b.lets {
            let v = self.eval(&l.value, env)?;
            env.push((l.name.clone(), v));
        }
        let r = match &b.terminal {
            Terminal::Return(e) => self.eval(e, env),
            Terminal::Using { body, .. } => self.body(body, env),
        };
        env.truncate(mark);
        r
    }

    fn block(&mut self, b: &Block, env: &mut Env) -> R {
        let mark = env.len();
        for l in &b.lets {
            let v = self.eval(&l.value, env)?;
            env.push((l.name.clone(), v));
        }
        let r = self.eval(&b.expr, env);
        env.truncate(mark);
        r
    }

    fn call_closure(&mut self, c: &Value, args: Vec<Value>) -> R {
        let Value::Closure(rc) = c else {
            return fault("not_callable");
        };
        let Some((params, body)) = self.lambdas.get(&rc.0).copied() else {
            return fault("not_callable");
        };
        let mut env = rc.1.clone();
        for ((n, _), v) in params.iter().zip(args) {
            env.push((n.clone(), v));
        }
        self.eval(body, &mut env)
    }

    fn eval(&mut self, e: &Expr, env: &mut Env) -> R {
        self.steps += 1;
        self.total += 1;
        if self.steps > self.max_steps {
            return fault("step_limit");
        }
        match &e.kind {
            ExprKind::Int(n) => {
                if self.info.byte_lits.contains(&e.id) {
                    Ok(Value::Byte(*n as u8))
                } else {
                    Ok(Value::Int(*n as i64))
                }
            }
            ExprKind::Float(x) => Ok(Value::Float(*x)),
            ExprKind::Str(s) => Ok(Value::str(s)),
            ExprKind::Bool(b) => Ok(Value::Bool(*b)),
            ExprKind::ResultKw => self.lookup(env, "result"),
            ExprKind::Var(n) => self.lookup(env, n),
            ExprKind::Call(name, args) => {
                let mut vs = Vec::new();
                for a in args {
                    vs.push(self.eval(a, env)?);
                }
                match name.as_str() {
                    _ if self.fns.contains_key(name.as_str()) => self.call_user(name, vs),
                    "approx" => match (&vs[0], &vs[1]) {
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(approx(*a, *b))),
                        _ => fault("type_error"),
                    },
                    // Output of `print` is discarded so that stdout only carries diagnostics.
                    "print" => Ok(Value::Bool(true)),
                    _ => fault("unknown_function"),
                }
            }
            ExprKind::Ctor(name, args) => {
                let mut vs = Vec::new();
                for a in args.iter().flatten() {
                    vs.push(self.eval(a, env)?);
                }
                Ok(Value::variant(name, vs))
            }
            ExprKind::Struct(name, fields) => {
                let mut vals: Vec<(String, Value)> = Vec::new();
                for (n, _, fe) in fields {
                    vals.push((n.clone(), self.eval(fe, env)?));
                }
                let mut ordered = Vec::new();
                if let Some(decl) = self.info.structs.get(name) {
                    for (fname, _) in decl {
                        if let Some(p) = vals.iter().position(|(n, _)| n == fname) {
                            ordered.push(vals.remove(p));
                        }
                    }
                }
                Ok(Value::Struct(Rc::from(name.as_str()), Rc::new(ordered)))
            }
            ExprKind::Method {
                recv, name, args, ..
            } => {
                let r = self.eval(recv, env)?;
                let mut vs = Vec::new();
                for a in args.iter().flatten() {
                    vs.push(self.eval(a, env)?);
                }
                self.method(e, r, name, vs, args.is_some())
            }
            ExprKind::Index(recv, idx) => self.index(recv, idx, env),
            ExprKind::Try(inner) => match self.eval(inner, env)? {
                Value::Variant(n, p) if &*n == "Ok" => Ok(p[0].clone()),
                v @ Value::Variant(..) => Err(Flow::Early(v)),
                _ => fault("type_error"),
            },
            ExprKind::Unary(UnOp::Neg, x) => match self.eval(x, env)? {
                Value::Int(i) => i
                    .checked_neg()
                    .map(Value::Int)
                    .ok_or(Flow::Fault("overflow".into())),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => fault("type_error"),
            },
            ExprKind::Unary(UnOp::Not, x) => match self.eval(x, env)? {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => fault("type_error"),
            },
            ExprKind::Binary(op, l, r) => self.binary(*op, l, r, env),
            ExprKind::Range(a, b, incl) => {
                let lo = self.eval(a, env)?;
                let hi = self.eval(b, env)?;
                Ok(Value::Range(Rc::new((lo, hi, *incl))))
            }
            ExprKind::List(items) => {
                let mut out = Vec::new();
                for i in items {
                    out.push(self.eval(i, env)?);
                }
                Ok(Value::list(out))
            }
            ExprKind::If(c, t, f) => match self.eval(c, env)? {
                Value::Bool(true) => self.block(t, env),
                Value::Bool(false) => self.block(f, env),
                _ => fault("type_error"),
            },
            ExprKind::Match(s, arms) => {
                let v = self.eval(s, env)?;
                for arm in arms {
                    let mark = env.len();
                    if self.matches(&arm.pat, &v, env) {
                        let r = self.eval(&arm.body, env);
                        env.truncate(mark);
                        return r;
                    }
                    env.truncate(mark);
                }
                fault("match_failure")
            }
            ExprKind::For {
                var, iter, body, ..
            } => {
                let it = self.eval(iter, env)?;
                let items: Vec<Value> = match &it {
                    Value::List(v) => v.to_vec(),
                    Value::Range(r) => match (&r.0, &r.1) {
                        (Value::Int(lo), Value::Int(hi)) => {
                            let end = if r.2 { hi.checked_add(1) } else { Some(*hi) };
                            let Some(end) = end else {
                                return fault("overflow");
                            };
                            if end.saturating_sub(*lo) > 5_000_000 {
                                return fault("step_limit");
                            }
                            (*lo..end).map(Value::Int).collect()
                        }
                        _ => return fault("type_error"),
                    },
                    _ => return fault("type_error"),
                };
                let mut out = Vec::new();
                for item in items {
                    env.push((var.clone(), item));
                    let r = self.block(body, env);
                    env.pop();
                    out.push(r?);
                }
                Ok(Value::list(out))
            }
            ExprKind::Lambda(..) => Ok(Value::Closure(Rc::new((e.id, env.clone())))),
        }
    }

    /// One evaluation step for one element of a lazily iterated range.
    fn tick(&mut self) -> Result<(), Flow> {
        self.steps += 1;
        self.total += 1;
        if self.steps > self.max_steps {
            return fault("step_limit");
        }
        Ok(())
    }

    /// Methods of `Range<Int>`: reductions iterate lazily (one step per element), `len` and
    /// `contains` are arithmetic, list-building methods respect the memory limit.
    fn range_method(&mut self, rg: &(Value, Value, bool), name: &str, args: Vec<Value>) -> R {
        let (Value::Int(lo), Value::Int(hi)) = (&rg.0, &rg.1) else {
            return fault("type_error");
        };
        let lo = i128::from(*lo);
        let end = i128::from(*hi) + i128::from(rg.2);
        let n = (end - lo).max(0);
        let int = |k: i128| Value::Int(k as i64);
        match name {
            "len" => i64::try_from(n)
                .map(Value::Int)
                .or_else(|_| fault("overflow")),
            "is_empty" => Ok(Value::Bool(n == 0)),
            "contains" => match &args[0] {
                Value::Int(x) => Ok(Value::Bool(lo <= i128::from(*x) && i128::from(*x) < end)),
                _ => fault("type_error"),
            },
            "sum" | "product" => {
                let mut acc: i64 = if name == "sum" { 0 } else { 1 };
                for k in lo..end {
                    self.tick()?;
                    let k = k as i64;
                    acc = match if name == "sum" {
                        acc.checked_add(k)
                    } else {
                        acc.checked_mul(k)
                    } {
                        Some(a) => a,
                        None => return fault("overflow"),
                    };
                }
                Ok(Value::Int(acc))
            }
            "min" | "max" => {
                if n == 0 {
                    return fault("empty_list");
                }
                for _ in 0..n.min(i128::from(self.max_steps) + 1) {
                    self.tick()?;
                }
                Ok(int(if name == "min" { lo } else { end - 1 }))
            }
            "count" => {
                let mut c = 0;
                for k in lo..end {
                    self.tick()?;
                    if args[0] == int(k) {
                        c += 1;
                    }
                }
                Ok(Value::Int(c))
            }
            "fold" => {
                let mut acc = args[0].clone();
                for k in lo..end {
                    self.tick()?;
                    acc = self.call_closure(&args[1], vec![acc, int(k)])?;
                }
                Ok(acc)
            }
            "any" | "all" => {
                let want_any = name == "any";
                for k in lo..end {
                    self.tick()?;
                    let hit = matches!(
                        self.call_closure(&args[0], vec![int(k)])?,
                        Value::Bool(true)
                    );
                    if hit == want_any {
                        return Ok(Value::Bool(want_any));
                    }
                }
                Ok(Value::Bool(!want_any))
            }
            "filter" => {
                let mut out = Vec::new();
                for k in lo..end {
                    self.tick()?;
                    if let Value::Bool(true) = self.call_closure(&args[0], vec![int(k)])? {
                        out.push(int(k));
                        if out.len() > MEMORY_LIMIT {
                            return fault("memory_limit");
                        }
                    }
                }
                Ok(Value::list(out))
            }
            "map" | "reverse" | "take" | "skip" => {
                let (mut a, mut b) = (lo, end.max(lo));
                if name == "take" || name == "skip" {
                    let Value::Int(k) = &args[0] else {
                        return fault("type_error");
                    };
                    let k = i128::from(*k).max(0);
                    if name == "take" {
                        b = b.min(a + k);
                    } else {
                        a = b.min(a + k);
                    }
                }
                // compare in i128 before any cast: a 2^64-element range must not truncate to 0
                if b - a > MEMORY_LIMIT as i128 {
                    return fault("memory_limit");
                }
                let len = (b - a) as usize;
                self.charge(len)?;
                let items: Vec<Value> = if name == "reverse" {
                    (a..b).rev().map(int).collect()
                } else {
                    (a..b).map(int).collect()
                };
                if name != "map" {
                    return Ok(Value::list(items));
                }
                let mut out = Vec::with_capacity(len);
                for x in items {
                    out.push(self.call_closure(&args[0], vec![x])?);
                }
                Ok(Value::list(out))
            }
            _ => fault("type_error"),
        }
    }

    /// Account for a freshly built value of `n` bytes/elements: over the limit is a fault, and
    /// big copies also consume step budget.
    fn charge(&mut self, n: usize) -> Result<(), Flow> {
        if n > MEMORY_LIMIT {
            return fault("memory_limit");
        }
        // one step per two bytes/elements copied: big copies drain the budgets quickly
        self.steps += (n >> 1) as u64;
        self.total += (n as u64).saturating_mul(4);
        Ok(())
    }

    fn index(&mut self, recv: &Expr, idx: &Expr, env: &mut Env) -> R {
        let r = self.eval(recv, env)?;
        let Value::List(v) = &r else {
            return fault("type_error");
        };
        if let ExprKind::Range(a, b, incl) = &idx.kind {
            let (Value::Int(lo), Value::Int(hi)) = (self.eval(a, env)?, self.eval(b, env)?) else {
                return fault("type_error");
            };
            let Some(hi) = (if *incl { hi.checked_add(1) } else { Some(hi) }) else {
                return fault("index_out_of_bounds");
            };
            if lo < 0 || lo > hi || hi as u64 > v.len() as u64 {
                return fault("index_out_of_bounds");
            }
            Ok(Value::list(v[lo as usize..hi as usize].to_vec()))
        } else {
            let Value::Int(i) = self.eval(idx, env)? else {
                return fault("type_error");
            };
            if i < 0 || i as u64 >= v.len() as u64 {
                return fault("index_out_of_bounds");
            }
            Ok(v[i as usize].clone())
        }
    }

    fn lookup(&self, env: &Env, name: &str) -> R {
        match env.iter().rev().find(|(n, _)| n == name) {
            Some((_, v)) => Ok(v.clone()),
            None => fault("unbound_name"),
        }
    }

    fn matches(&mut self, p: &Pattern, v: &Value, env: &mut Env) -> bool {
        match (&p.kind, v) {
            (PatKind::Wild, _) => true,
            (PatKind::Bind(n), _) => {
                env.push((n.clone(), v.clone()));
                true
            }
            (PatKind::Int(n), Value::Int(i)) => n == i,
            (PatKind::Int(n), Value::Byte(b)) => *n == i64::from(*b),
            (PatKind::Float(x), Value::Float(y)) => x == y,
            (PatKind::Str(s), Value::Str(t)) => **s == **t,
            (PatKind::Bool(a), Value::Bool(b)) => a == b,
            (PatKind::Ctor(n, subs), Value::Variant(vn, payload)) => {
                **n == **vn
                    && subs.len() == payload.len()
                    && subs
                        .iter()
                        .zip(payload.iter())
                        .all(|(sp, sv)| self.matches(sp, sv, env))
            }
            _ => false,
        }
    }

    fn binary(&mut self, op: BinOp, l: &Expr, r: &Expr, env: &mut Env) -> R {
        match op {
            BinOp::And => {
                return match self.eval(l, env)? {
                    Value::Bool(false) => Ok(Value::Bool(false)),
                    Value::Bool(true) => self.eval(r, env),
                    _ => fault("type_error"),
                }
            }
            BinOp::Or => {
                return match self.eval(l, env)? {
                    Value::Bool(true) => Ok(Value::Bool(true)),
                    Value::Bool(false) => self.eval(r, env),
                    _ => fault("type_error"),
                }
            }
            _ => {}
        }
        let a = self.eval(l, env)?;
        let b = self.eval(r, env)?;
        if let (BinOp::Add, Value::Str(x), Value::Str(y)) = (op, &a, &b) {
            self.charge(x.len().saturating_add(y.len()))?;
        }
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => arith(op, &a, &b),
            BinOp::Eq => Ok(Value::Bool(a == b)),
            BinOp::Ne => Ok(Value::Bool(a != b)),
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                let o = cmp_values(&a, &b);
                Ok(Value::Bool(match (op, o) {
                    (_, None) => false,
                    (BinOp::Lt, Some(o)) => o == Ordering::Less,
                    (BinOp::Le, Some(o)) => o != Ordering::Greater,
                    (BinOp::Gt, Some(o)) => o == Ordering::Greater,
                    (_, Some(o)) => o != Ordering::Less,
                }))
            }
            BinOp::In => match &b {
                Value::Range(rg) => {
                    let lo = cmp_values(&a, &rg.0);
                    let hi = cmp_values(&a, &rg.1);
                    Ok(Value::Bool(match (lo, hi) {
                        (Some(lo), Some(hi)) => {
                            lo != Ordering::Less
                                && if rg.2 {
                                    hi != Ordering::Greater
                                } else {
                                    hi == Ordering::Less
                                }
                        }
                        _ => false,
                    }))
                }
                Value::List(v) => Ok(Value::Bool(v.contains(&a))),
                _ => fault("type_error"),
            },
            BinOp::And | BinOp::Or => unreachable!(),
        }
    }

    fn method(&mut self, e: &Expr, recv: Value, name: &str, args: Vec<Value>, called: bool) -> R {
        if !called {
            if let Value::Struct(_, fields) = &recv {
                return match fields.iter().find(|(n, _)| n == name) {
                    Some((_, v)) => Ok(v.clone()),
                    None => fault("type_error"),
                };
            }
            return fault("type_error");
        }
        if let Value::Range(rg) = &recv {
            return self.range_method(rg, name, args);
        }
        match (&recv, name) {
            (Value::List(v), "len") => Ok(Value::Int(v.len() as i64)),
            (Value::Str(s), "len") => Ok(Value::Int(s.len() as i64)),
            (Value::List(v), "sum") => {
                if v.is_empty() {
                    return Ok(if self.info.float_sums.contains(&e.id) {
                        Value::Float(0.0)
                    } else {
                        Value::Int(0)
                    });
                }
                match &v[0] {
                    Value::Int(_) => {
                        let mut acc: i64 = 0;
                        for x in v.iter() {
                            if let Value::Int(i) = x {
                                acc = match acc.checked_add(*i) {
                                    Some(a) => a,
                                    None => return fault("overflow"),
                                };
                            }
                        }
                        Ok(Value::Int(acc))
                    }
                    _ => {
                        let mut acc = 0.0;
                        for x in v.iter() {
                            if let Value::Float(f) = x {
                                acc += *f;
                            }
                        }
                        Ok(Value::Float(acc))
                    }
                }
            }
            (Value::List(v), "min") | (Value::List(v), "max") => {
                if v.is_empty() {
                    return fault("empty_list");
                }
                let mut best = v[0].clone();
                for x in v.iter().skip(1) {
                    let o = total_cmp(x, &best);
                    if (name == "min" && o == Ordering::Less)
                        || (name == "max" && o == Ordering::Greater)
                    {
                        best = x.clone();
                    }
                }
                Ok(best)
            }
            (Value::List(v), "contains") => Ok(Value::Bool(v.contains(&args[0]))),
            (Value::List(v), "count") => Ok(Value::Int(
                v.iter().filter(|x| **x == args[0]).count() as i64,
            )),
            (Value::List(v), "reverse") => Ok(Value::list(v.iter().rev().cloned().collect())),
            (Value::List(v), "sort") => {
                let mut out = v.to_vec();
                out.sort_by(total_cmp);
                Ok(Value::list(out))
            }
            (Value::List(v), "push") => {
                self.charge(v.len() + 1)?;
                let mut out = v.to_vec();
                out.push(args[0].clone());
                Ok(Value::list(out))
            }
            (Value::List(v), "map") => {
                let mut out = Vec::new();
                for x in v.iter() {
                    out.push(self.call_closure(&args[0], vec![x.clone()])?);
                }
                Ok(Value::list(out))
            }
            (Value::List(v), "filter") => {
                let mut out = Vec::new();
                for x in v.iter() {
                    if let Value::Bool(true) = self.call_closure(&args[0], vec![x.clone()])? {
                        out.push(x.clone());
                    }
                }
                Ok(Value::list(out))
            }
            (Value::List(v), "fold") => {
                let mut acc = args[0].clone();
                for x in v.iter() {
                    acc = self.call_closure(&args[1], vec![acc, x.clone()])?;
                }
                Ok(acc)
            }
            (Value::List(v), "is_empty") => Ok(Value::Bool(v.is_empty())),
            (Value::Str(s), "to_string") => Ok(Value::Str(s.clone())),
            (Value::Str(s), "is_empty") => Ok(Value::Bool(s.is_empty())),
            (Value::List(v), "any") | (Value::List(v), "all") => {
                let want_any = name == "any";
                for x in v.iter() {
                    let hit = matches!(
                        self.call_closure(&args[0], vec![x.clone()])?,
                        Value::Bool(true)
                    );
                    if hit == want_any {
                        return Ok(Value::Bool(want_any));
                    }
                }
                Ok(Value::Bool(!want_any))
            }
            (Value::List(v), "take") | (Value::List(v), "skip") => {
                let Value::Int(n) = &args[0] else {
                    return fault("type_error");
                };
                let n = (*n).max(0) as u64;
                let n = n.min(v.len() as u64) as usize;
                Ok(Value::list(if name == "take" {
                    v[..n].to_vec()
                } else {
                    v[n..].to_vec()
                }))
            }
            (Value::List(v), "concat") => match &args[0] {
                Value::List(w) => {
                    self.charge(v.len().saturating_add(w.len()))?;
                    let mut out = v.to_vec();
                    out.extend(w.iter().cloned());
                    Ok(Value::list(out))
                }
                _ => fault("type_error"),
            },
            (Value::List(v), "dedup") => {
                let mut out: Vec<Value> = Vec::new();
                for x in v.iter() {
                    if out.last() != Some(x) {
                        out.push(x.clone());
                    }
                }
                Ok(Value::list(out))
            }
            (Value::List(v), "product") => {
                if v.is_empty() {
                    return Ok(if self.info.float_sums.contains(&e.id) {
                        Value::Float(1.0)
                    } else {
                        Value::Int(1)
                    });
                }
                if let Value::Int(_) = &v[0] {
                    let mut acc: i64 = 1;
                    for x in v.iter() {
                        if let Value::Int(i) = x {
                            acc = match acc.checked_mul(*i) {
                                Some(a) => a,
                                None => return fault("overflow"),
                            };
                        }
                    }
                    Ok(Value::Int(acc))
                } else {
                    let mut acc = 1.0;
                    for x in v.iter() {
                        if let Value::Float(f) = x {
                            acc *= *f;
                        }
                    }
                    Ok(Value::Float(acc))
                }
            }
            (Value::List(v), "join") => {
                let Value::Str(sep) = &args[0] else {
                    return fault("type_error");
                };
                let mut parts: Vec<&str> = Vec::new();
                for x in v.iter() {
                    match x {
                        Value::Str(s) => parts.push(s),
                        _ => return fault("type_error"),
                    }
                }
                let total: usize = parts.iter().map(|p| p.len()).sum::<usize>()
                    + sep.len().saturating_mul(parts.len());
                self.charge(total)?;
                Ok(Value::str(&parts.join(sep)))
            }
            (Value::Int(a), "min") | (Value::Int(a), "max") | (Value::Int(a), "pow") => {
                let Value::Int(b) = &args[0] else {
                    return fault("type_error");
                };
                match name {
                    "min" => Ok(Value::Int(*a.min(b))),
                    "max" => Ok(Value::Int(*a.max(b))),
                    _ => int_pow(*a, *b),
                }
            }
            (Value::Int(i), "to_string") => Ok(Value::str(&i.to_string())),
            (Value::Byte(b), "to_string") => Ok(Value::str(&b.to_string())),
            (Value::Bool(b), "to_string") => Ok(Value::str(&b.to_string())),
            (Value::Float(f), "to_string") => Ok(Value::str(&f.to_string())),
            (Value::Float(f), "sqrt") => Ok(Value::Float(f.sqrt())),
            (Value::Float(f), "floor") => Ok(Value::Float(f.floor())),
            (Value::Float(f), "ceil") => Ok(Value::Float(f.ceil())),
            (Value::Float(f), "round") => Ok(Value::Float(f.round())),
            (Value::Float(a), "min") | (Value::Float(a), "max") => {
                let Value::Float(b) = &args[0] else {
                    return fault("type_error");
                };
                Ok(Value::Float(if name == "min" {
                    a.min(*b)
                } else {
                    a.max(*b)
                }))
            }
            (Value::Float(f), "int") => {
                if f.is_nan()
                    || *f >= 9_223_372_036_854_775_808.0
                    || *f < -9_223_372_036_854_775_808.0
                {
                    fault("overflow")
                } else {
                    Ok(Value::Int(f.trunc() as i64))
                }
            }
            (Value::Str(s), "chars") => Ok(Value::list(
                s.chars().map(|c| Value::str(&c.to_string())).collect(),
            )),
            (Value::Str(s), "contains")
            | (Value::Str(s), "starts_with")
            | (Value::Str(s), "ends_with") => {
                let Value::Str(t) = &args[0] else {
                    return fault("type_error");
                };
                Ok(Value::Bool(match name {
                    "contains" => s.contains(&**t),
                    "starts_with" => s.starts_with(&**t),
                    _ => s.ends_with(&**t),
                }))
            }
            (Value::Str(s), "split") => {
                let Value::Str(t) = &args[0] else {
                    return fault("type_error");
                };
                Ok(Value::list(s.split(&**t).map(Value::str).collect()))
            }
            (Value::Str(s), "trim") => Ok(Value::str(s.trim())),
            (Value::Str(s), "to_uppercase") => Ok(Value::str(&s.to_uppercase())),
            (Value::Str(s), "to_lowercase") => Ok(Value::str(&s.to_lowercase())),
            (Value::Str(s), "parse_int") => Ok(parse_int(s)),
            (Value::Int(i), "float") => Ok(Value::Float(*i as f64)),
            (Value::Byte(b), "float") => Ok(Value::Float(f64::from(*b))),
            (Value::Byte(b), "int") => Ok(Value::Int(i64::from(*b))),
            (Value::Int(i), "abs") => i
                .checked_abs()
                .map(Value::Int)
                .ok_or(Flow::Fault("overflow".into())),
            (Value::Float(f), "abs") => Ok(Value::Float(f.abs())),
            (Value::Variant(n, _), "is_ok") => Ok(Value::Bool(&**n == "Ok")),
            (Value::Variant(n, _), "is_err") => Ok(Value::Bool(&**n == "Err")),
            (Value::Variant(n, p), "unwrap") => {
                if &**n == "Ok" {
                    Ok(p[0].clone())
                } else {
                    fault("unwrap_on_err")
                }
            }
            _ => fault("type_error"),
        }
    }
}

fn arith(op: BinOp, a: &Value, b: &Value) -> R {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => {
            let r = match op {
                BinOp::Add => x.checked_add(*y),
                BinOp::Sub => x.checked_sub(*y),
                BinOp::Mul => x.checked_mul(*y),
                BinOp::Div | BinOp::Rem => {
                    if *y == 0 {
                        return fault("division_by_zero");
                    }
                    if op == BinOp::Div {
                        x.checked_div(*y)
                    } else {
                        x.checked_rem(*y)
                    }
                }
                _ => None,
            };
            r.map(Value::Int).ok_or(Flow::Fault("overflow".into()))
        }
        (Value::Byte(x), Value::Byte(y)) => {
            let r = match op {
                BinOp::Add => x.checked_add(*y),
                BinOp::Sub => x.checked_sub(*y),
                BinOp::Mul => x.checked_mul(*y),
                BinOp::Div | BinOp::Rem => {
                    if *y == 0 {
                        return fault("division_by_zero");
                    }
                    if op == BinOp::Div {
                        Some(x / y)
                    } else {
                        Some(x % y)
                    }
                }
                _ => None,
            };
            r.map(Value::Byte).ok_or(Flow::Fault("overflow".into()))
        }
        (Value::Str(x), Value::Str(y)) if op == BinOp::Add => {
            let mut out = String::with_capacity(x.len() + y.len());
            out.push_str(x);
            out.push_str(y);
            Ok(Value::str(&out))
        }
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(match op {
            BinOp::Add => x + y,
            BinOp::Sub => x - y,
            BinOp::Mul => x * y,
            BinOp::Div => x / y,
            _ => x % y,
        })),
        _ => fault("type_error"),
    }
}

fn int_pow(base: i64, exp: i64) -> R {
    if exp < 0 {
        return fault("negative_exponent");
    }
    let mut result: i64 = 1;
    let mut b = base;
    let mut e = exp as u64;
    while e > 0 {
        if e & 1 == 1 {
            result = match result.checked_mul(b) {
                Some(r) => r,
                None => return fault("overflow"),
            };
        }
        e >>= 1;
        if e > 0 {
            b = match b.checked_mul(b) {
                Some(r) => r,
                None => return fault("overflow"),
            };
        }
    }
    Ok(Value::Int(result))
}

/// `String -> Result<Int, String>`: optional sign, then decimal digits.
fn parse_int(s: &str) -> Value {
    if s.is_empty() {
        return Value::err(Value::str("empty string"));
    }
    let digits = s.strip_prefix(['+', '-']).unwrap_or(s);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Value::err(Value::str("invalid digit"));
    }
    match s.parse::<i64>() {
        Ok(n) => Value::ok(Value::Int(n)),
        Err(_) if s.starts_with('-') => Value::err(Value::str("number too small")),
        Err(_) => Value::err(Value::str("number too large")),
    }
}

/// Total order on comparable values (used for sorting edge cases).
pub fn interp_cmp(a: &Value, b: &Value) -> Ordering {
    total_cmp(a, b)
}
