//! Per-function checking: names, bidirectional types, effects and spec rules.

use crate::exh::{self, Pat};
use crate::info::{resolve_type, ModuleInfo};
use crate::prelude;
use crate::ty::{unify, Ty};
use aval_syntax::ast::*;
use aval_syntax::diag::{Diag, Pos};
use std::collections::BTreeSet;

#[derive(Clone, Copy, PartialEq)]
enum Need {
    Numeric,
    /// `+`: numeric or `String`.
    Add,
    Ordered,
    Any,
}

pub struct Fc<'a> {
    src: &'a str,
    info: &'a ModuleInfo,
    types: BTreeSet<String>,
    fname: String,
    ret: Ty,
    scopes: Vec<Vec<(String, Ty)>>,
    declared: BTreeSet<String>,
    in_ensures: bool,
    allow_result: bool,
    pure_spec: bool,
    effect_calls: usize,
    /// inside `examples`: only literals, constructors and prelude functions are in scope
    closed: bool,
    pub diags: Vec<Diag>,
    pub byte_lits: BTreeSet<u32>,
    pub float_sums: BTreeSet<u32>,
}

fn is_int_lit(e: &Expr) -> bool {
    matches!(e.kind, ExprKind::Int(_))
}

fn sig_string(params: &[Ty], ret: &Ty) -> String {
    let ps: Vec<String> = params.iter().map(|p| p.to_string()).collect();
    format!("fn({}) -> {}", ps.join(", "), ret)
}

impl<'a> Fc<'a> {
    pub fn new(src: &'a str, info: &'a ModuleInfo, f: &FnDecl, ret: Ty) -> Fc<'a> {
        Fc {
            src,
            info,
            types: info.type_names(),
            fname: f.name.clone(),
            ret,
            scopes: vec![Vec::new()],
            declared: f
                .effects
                .as_ref()
                .map(|e| e.names.iter().map(|(n, _)| n.clone()).collect())
                .unwrap_or_default(),
            in_ensures: false,
            allow_result: false,
            pure_spec: false,
            effect_calls: 0,
            closed: false,
            diags: Vec::new(),
            byte_lits: BTreeSet::new(),
            float_sums: BTreeSet::new(),
        }
    }

    fn push(&mut self, d: Diag) {
        let d = d.with_fn(&self.fname.clone());
        self.diags.push(d);
    }

    fn mismatch(&mut self, pos: Pos, exp: &Ty, found: &Ty) {
        self.push(
            Diag::new("E301", pos, "type mismatch")
                .with("expected", exp.to_string())
                .with("found", found.to_string()),
        );
    }

    fn arity_err(&mut self, pos: Pos, expected: usize, found: usize) {
        self.push(
            Diag::new("E303", pos, "wrong number of arguments")
                .with("expected", expected)
                .with("found", found),
        );
    }

    fn lookup(&self, name: &str) -> Option<Ty> {
        for s in self.scopes.iter().rev() {
            if let Some((_, t)) = s.iter().rev().find(|(n, _)| n == name) {
                return Some(t.clone());
            }
        }
        None
    }

    fn declare(&mut self, name: &str, pos: Pos, ty: Ty) {
        if self.lookup(name).is_some() {
            self.push(
                Diag::new(
                    "E102",
                    pos,
                    "name already declared (shadowing is forbidden)",
                )
                .with("name", name),
            );
        }
        if let Some(s) = self.scopes.last_mut() {
            s.push((name.to_string(), ty));
        }
    }

    fn ty(&mut self, te: &TypeExpr) -> Ty {
        let mut sink = Vec::new();
        let t = resolve_type(te, &self.types, &self.info.broken, &mut sink, None);
        for d in sink {
            self.push(d);
        }
        t
    }

    fn finish(&mut self, e: &Expr, exp: Option<&Ty>, t: Ty) -> Ty {
        match exp {
            None => t,
            Some(x) => match unify(x, &t) {
                Some(u) => u,
                None => {
                    self.mismatch(e.pos, x, &t);
                    x.clone()
                }
            },
        }
    }

    fn call_effects(&mut self, pos: Pos, effects: &[String]) {
        for eff in effects {
            self.effect_calls += 1;
            if self.pure_spec || !self.declared.contains(eff) {
                self.push(
                    Diag::new("E420", pos, "effect not declared in `effects`")
                        .with("effect", eff.as_str()),
                );
            }
        }
    }

    // ---- function level ---------------------------------------------------

    pub fn check_fn(&mut self, it: &Item, f: &FnDecl, params: &[(String, Ty)]) {
        for (p, (_, pty)) in f.params.iter().zip(params) {
            self.declare(&p.name, p.pos, pty.clone());
        }
        self.pure_spec = true;
        for c in &f.requires {
            self.expr(&c.expr, Some(&Ty::Bool));
        }
        self.in_ensures = true;
        self.allow_result = true;
        for c in &f.ensures {
            self.expr(&c.expr, Some(&Ty::Bool));
        }
        self.in_ensures = false;
        self.allow_result = false;
        let saved_scopes = std::mem::replace(&mut self.scopes, vec![Vec::new()]);
        self.closed = true;
        for ex in &f.examples {
            if ex.inputs.len() != params.len() {
                self.arity_err(ex.pos, params.len(), ex.inputs.len());
                for i in &ex.inputs {
                    self.expr(i, None);
                }
            } else {
                for (i, (_, pt)) in ex.inputs.iter().zip(params) {
                    self.expr(i, Some(pt));
                }
            }
            let ret = self.ret.clone();
            self.expr(&ex.output, Some(&ret));
        }
        self.closed = false;
        self.scopes = saved_scopes;
        self.pure_spec = false;
        if let Some((name, pos)) = &f.oracle {
            match self.info.fns.get(name) {
                None => self.push(
                    Diag::new("E101", *pos, "unknown oracle function").with("name", name.as_str()),
                ),
                Some(_) if *name == f.name => self.push(
                    Diag::new("E107", *pos, "a function cannot be its own oracle")
                        .with("name", name.as_str()),
                ),
                Some(g) if !g.effects.is_empty() => self.push(
                    Diag::new("E107", *pos, "an oracle must be pure").with("name", name.as_str()),
                ),
                Some(g) => {
                    let mine: Vec<Ty> = params.iter().map(|(_, t)| t.clone()).collect();
                    let theirs: Vec<Ty> = g.params.iter().map(|(_, t)| t.clone()).collect();
                    if mine != theirs || g.ret != self.ret {
                        let (e, found) =
                            (sig_string(&mine, &self.ret), sig_string(&theirs, &g.ret));
                        self.push(
                            Diag::new("E301", *pos, "oracle signature differs")
                                .with("expected", e)
                                .with("found", found),
                        );
                    }
                }
            }
        }
        if let Some(a) = &f.alloc {
            if let Some((n, p)) = &a.reuse {
                if self.lookup(n).is_none() {
                    self.push(Diag::new("E101", *p, "unknown name").with("name", n.as_str()));
                }
            }
        }
        self.body(&f.body);
        if it.is_pub {
            if f.ensures.is_empty() && !f.has_examples {
                self.push(Diag::new(
                    "E510",
                    it.pos,
                    "pub function needs `ensures` or `examples`",
                ));
            }
            if f.cost.is_none() {
                self.push(Diag::new(
                    "E511",
                    it.pos,
                    "pub function needs a `cost` clause",
                ));
            }
        }
    }

    fn body(&mut self, b: &Body) {
        self.scopes.push(Vec::new());
        self.lets(&b.lets);
        match &b.terminal {
            Terminal::Return(e) => {
                let ret = self.ret.clone();
                self.expr(e, Some(&ret));
            }
            Terminal::Using { name, pos, body } => {
                if self.lookup(name).is_none() {
                    self.push(Diag::new("E101", *pos, "unknown name").with("name", name.as_str()));
                }
                self.body(body);
            }
        }
        self.scopes.pop();
    }

    fn lets(&mut self, lets: &[Let]) {
        for l in lets {
            let t = self.ty(&l.ty);
            self.expr(&l.value, Some(&t));
            self.declare(&l.name, l.pos, t);
        }
    }

    fn block(&mut self, b: &Block, exp: Option<&Ty>) -> Ty {
        self.scopes.push(Vec::new());
        self.lets(&b.lets);
        let t = self.expr(&b.expr, exp);
        self.scopes.pop();
        t
    }

    // ---- expressions --------------------------------------------------------

    pub fn expr(&mut self, e: &Expr, exp: Option<&Ty>) -> Ty {
        let t = match &e.kind {
            ExprKind::Int(n) => {
                if matches!(exp, Some(Ty::Byte)) {
                    if *n > 255 {
                        self.push(
                            Diag::new("E302", e.pos, "literal out of range for Byte")
                                .with("found", *n)
                                .with("expected", "0..=255"),
                        );
                    } else {
                        self.byte_lits.insert(e.id);
                    }
                    return Ty::Byte;
                }
                Ty::Int
            }
            ExprKind::Float(_) => Ty::Float,
            ExprKind::Str(_) => Ty::Str,
            ExprKind::Bool(_) => Ty::Bool,
            ExprKind::ResultKw => {
                if self.allow_result {
                    self.ret.clone()
                } else {
                    self.push(Diag::new(
                        "E106",
                        e.pos,
                        "`result` is only valid inside `ensures`",
                    ));
                    Ty::Any
                }
            }
            ExprKind::Var(name) => match self.lookup(name) {
                Some(t) => t,
                None => {
                    self.push(Diag::new("E101", e.pos, "unknown name").with("name", name.as_str()));
                    Ty::Any
                }
            },
            ExprKind::Call(name, args) => self.call(e, name, args),
            ExprKind::Ctor(name, args) => self.ctor(e, name, args.as_deref(), exp),
            ExprKind::Struct(name, fields) => self.struct_lit(e, name, fields),
            ExprKind::Method {
                recv,
                name,
                name_end,
                args,
            } => self.method(e, recv, name, *name_end, args.as_deref(), exp),
            ExprKind::Try(inner) => self.try_expr(e, inner),
            ExprKind::Index(recv, idx) => self.index(e, recv, idx),
            ExprKind::Unary(UnOp::Neg, x) => {
                let hint = match exp {
                    Some(t @ (Ty::Int | Ty::Float)) => Some(t),
                    _ => None,
                };
                let t = self.expr(x, hint);
                if matches!(t, Ty::Int | Ty::Float | Ty::Any) {
                    t
                } else {
                    self.mismatch(x.pos, &Ty::Int, &t);
                    Ty::Any
                }
            }
            ExprKind::Unary(UnOp::Not, x) => {
                self.expr(x, Some(&Ty::Bool));
                Ty::Bool
            }
            ExprKind::Binary(op, l, r) => self.binary(e, *op, l, r, exp),
            ExprKind::Range(a, b, _) => {
                let hint = match exp {
                    Some(Ty::Range(t)) => Some((**t).clone()),
                    _ => None,
                };
                let (lt, _) = self.operands(a, b, hint.as_ref(), Need::Numeric);
                Ty::Range(Box::new(lt))
            }
            ExprKind::List(items) => {
                if let Some(Ty::List(et)) = exp {
                    let mut elem = (**et).clone();
                    for i in items {
                        let t = self.expr(i, Some(&elem));
                        elem = unify(&elem, &t).unwrap_or(elem);
                    }
                    Ty::List(Box::new(elem))
                } else {
                    let mut elem = Ty::Any;
                    for (k, i) in items.iter().enumerate() {
                        if k == 0 {
                            elem = self.expr(i, None);
                        } else {
                            let t = self.expr(i, Some(&elem));
                            elem = unify(&elem, &t).unwrap_or(elem);
                        }
                    }
                    Ty::List(Box::new(elem))
                }
            }
            ExprKind::If(c, t, f) => {
                self.expr(c, Some(&Ty::Bool));
                let tt = self.block(t, exp);
                let ft = if exp.is_some() {
                    self.block(f, exp)
                } else {
                    self.block(f, Some(&tt))
                };
                unify(&tt, &ft).unwrap_or(tt)
            }
            ExprKind::Match(s, arms) => self.match_expr(e, s, arms, exp),
            ExprKind::For {
                par,
                var,
                var_pos,
                iter,
                body,
            } => {
                let it = self.expr(iter, None);
                let elem = match it {
                    Ty::List(t) => *t,
                    Ty::Range(t) if matches!(*t, Ty::Int | Ty::Any) => *t,
                    Ty::Any => Ty::Any,
                    other => {
                        self.mismatch(iter.pos, &Ty::List(Box::new(Ty::Any)), &other);
                        Ty::Any
                    }
                };
                let exp_elem = match exp {
                    Some(Ty::List(t)) => Some((**t).clone()),
                    _ => None,
                };
                let before = self.effect_calls;
                self.scopes.push(Vec::new());
                self.declare(var, *var_pos, elem);
                let bt = self.block(body, exp_elem.as_ref());
                self.scopes.pop();
                if *par && self.effect_calls > before {
                    self.push(Diag::new("E421", e.pos, "`par for` body must be pure"));
                }
                Ty::List(Box::new(bt))
            }
            ExprKind::Lambda(params, body) => {
                let (ps, r) = match exp {
                    Some(Ty::Fn(ps, r)) => (ps.clone(), (**r).clone()),
                    _ => (vec![Ty::Any; params.len()], Ty::Any),
                };
                if ps.len() != params.len() {
                    self.arity_err(e.pos, ps.len(), params.len());
                }
                self.scopes.push(Vec::new());
                for (i, (n, p)) in params.iter().enumerate() {
                    self.declare(n, *p, ps.get(i).cloned().unwrap_or(Ty::Any));
                }
                let bt = if r == Ty::Any {
                    self.expr(body, None)
                } else {
                    self.expr(body, Some(&r))
                };
                self.scopes.pop();
                Ty::Fn(ps, Box::new(bt))
            }
        };
        self.finish(e, exp, t)
    }

    fn call(&mut self, e: &Expr, name: &str, args: &[Expr]) -> Ty {
        let user = if self.closed {
            None
        } else {
            self.info.fns.get(name).cloned()
        };
        if let Some(sig) = user {
            if args.len() != sig.params.len() {
                self.arity_err(e.pos, sig.params.len(), args.len());
            }
            for (k, a) in args.iter().enumerate() {
                let pt = sig.params.get(k).map(|(_, t)| t.clone());
                self.expr(a, pt.as_ref());
            }
            self.call_effects(e.pos, &sig.effects);
            return sig.ret;
        }
        if let Some((arity, effects)) = prelude::function(name) {
            if args.len() != arity {
                self.arity_err(e.pos, arity, args.len());
            }
            for (k, a) in args.iter().enumerate() {
                let pt = if name == "approx" { Ty::Float } else { Ty::Str };
                self.expr(a, if k < arity { Some(&pt) } else { None });
            }
            let effs: Vec<String> = effects.iter().map(|s| s.to_string()).collect();
            self.call_effects(e.pos, &effs);
            return Ty::Bool;
        }
        if !self.info.broken.contains(name) {
            let msg = if self.closed {
                "not in scope in `examples`"
            } else {
                "unknown function"
            };
            self.push(Diag::new("E101", e.pos, msg).with("name", name));
        }
        for a in args {
            self.expr(a, None);
        }
        Ty::Any
    }

    fn ctor(&mut self, e: &Expr, name: &str, args: Option<&[Expr]>, exp: Option<&Ty>) -> Ty {
        let argv = args.unwrap_or(&[]);
        if name == "Ok" || name == "Err" {
            if argv.len() != 1 {
                self.arity_err(e.pos, 1, argv.len());
                for a in argv {
                    self.expr(a, None);
                }
                return Ty::Result(Box::new(Ty::Any), Box::new(Ty::Any));
            }
            let (et, ee) = match exp {
                Some(Ty::Result(t, er)) => (Some((**t).clone()), Some((**er).clone())),
                _ => (None, None),
            };
            return if name == "Ok" {
                let t = self.expr(&argv[0], et.as_ref());
                Ty::Result(Box::new(t), Box::new(ee.unwrap_or(Ty::Any)))
            } else {
                let t = self.expr(&argv[0], ee.as_ref());
                Ty::Result(Box::new(et.unwrap_or(Ty::Any)), Box::new(t))
            };
        }
        if let Some(en) = self.info.variants.get(name).cloned() {
            let payload = self
                .info
                .enums
                .get(&en)
                .and_then(|vs| vs.iter().find(|(n, _)| n == name))
                .map(|(_, p)| p.clone())
                .unwrap_or_default();
            if argv.len() != payload.len() {
                self.arity_err(e.pos, payload.len(), argv.len());
            }
            for (k, a) in argv.iter().enumerate() {
                self.expr(a, payload.get(k));
            }
            return Ty::User(en);
        }
        if !self.info.broken.contains(name) {
            self.push(Diag::new("E101", e.pos, "unknown constructor").with("name", name));
        }
        for a in argv {
            self.expr(a, None);
        }
        Ty::Any
    }

    fn struct_lit(&mut self, e: &Expr, name: &str, fields: &[(String, Pos, Expr)]) -> Ty {
        let Some(decl) = self.info.structs.get(name).cloned() else {
            if !self.info.broken.contains(name) {
                self.push(Diag::new("E101", e.pos, "unknown type").with("name", name));
            }
            for (_, _, fe) in fields {
                self.expr(fe, None);
            }
            return Ty::Any;
        };
        let mut given: Vec<&str> = Vec::new();
        for (fname, fpos, fe) in fields {
            match decl.iter().find(|(n, _)| n == fname) {
                Some((_, fty)) => {
                    if given.contains(&fname.as_str()) {
                        self.push(
                            Diag::new("E102", *fpos, "duplicate field")
                                .with("name", fname.as_str()),
                        );
                    }
                    given.push(fname);
                    self.expr(fe, Some(fty));
                }
                None => {
                    let mut avail: Vec<String> = decl.iter().map(|(n, _)| n.clone()).collect();
                    avail.sort();
                    self.push(
                        Diag::new("E204", *fpos, "unknown field")
                            .with("found", fname.as_str())
                            .with("on", name)
                            .with("avail", avail),
                    );
                    self.expr(fe, None);
                }
            }
        }
        for (n, _) in &decl {
            if !given.contains(&n.as_str()) {
                self.push(
                    Diag::new("E303", e.pos, format!("missing field `{}`", n))
                        .with("expected", decl.len())
                        .with("found", fields.len()),
                );
            }
        }
        Ty::User(name.to_string())
    }

    fn try_expr(&mut self, e: &Expr, inner: &Expr) -> Ty {
        let it = self.expr(inner, None);
        match it {
            Ty::Any => Ty::Any,
            Ty::Result(t, err) => {
                match self.ret.clone() {
                    Ty::Result(_, fe) => {
                        if unify(&err, &fe).is_none() {
                            self.mismatch(
                                e.pos,
                                &Ty::Result(Box::new(Ty::Any), fe),
                                &Ty::Result(t.clone(), err),
                            );
                        }
                    }
                    Ty::Any => {}
                    other => {
                        let exp = Ty::Result(Box::new(Ty::Any), err);
                        self.mismatch(e.pos, &exp, &other);
                    }
                }
                *t
            }
            other => {
                let exp = Ty::Result(Box::new(Ty::Any), Box::new(Ty::Any));
                self.mismatch(inner.pos, &exp, &other);
                Ty::Any
            }
        }
    }

    fn operands(&mut self, l: &Expr, r: &Expr, hint: Option<&Ty>, need: Need) -> (Ty, Ty) {
        let ok = |t: &Ty| match need {
            Need::Numeric => t.is_numeric(),
            Need::Add => t.is_numeric() || *t == Ty::Str,
            Need::Ordered => t.is_ordered(),
            Need::Any => true,
        };
        if is_int_lit(l) && !is_int_lit(r) {
            let rt = self.expr(r, hint);
            if !ok(&rt) {
                self.mismatch(r.pos, &Ty::Int, &rt);
                self.expr(l, None);
                return (Ty::Any, Ty::Any);
            }
            let lt = self.expr(l, Some(&rt));
            (lt, rt)
        } else {
            let lt = self.expr(l, hint);
            if !ok(&lt) {
                self.mismatch(l.pos, &Ty::Int, &lt);
                self.expr(r, None);
                return (Ty::Any, Ty::Any);
            }
            let rt = self.expr(r, Some(&lt));
            (lt, rt)
        }
    }

    fn binary(&mut self, e: &Expr, op: BinOp, l: &Expr, r: &Expr, exp: Option<&Ty>) -> Ty {
        match op {
            BinOp::And | BinOp::Or => {
                self.expr(l, Some(&Ty::Bool));
                self.expr(r, Some(&Ty::Bool));
                Ty::Bool
            }
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                let hint = match exp {
                    Some(t @ (Ty::Int | Ty::Float | Ty::Byte)) => Some(t),
                    _ => None,
                };
                let need = if op == BinOp::Add {
                    Need::Add
                } else {
                    Need::Numeric
                };
                let before = self.diags.len();
                let (lt, rt) = self.operands(l, r, hint, need);
                if op == BinOp::Add && self.diags.len() > before {
                    // the operands already reported their mismatch: do not cascade
                    return Ty::Any;
                }
                unify(&lt, &rt).unwrap_or(lt)
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                self.operands(l, r, None, Need::Ordered);
                Ty::Bool
            }
            BinOp::Eq | BinOp::Ne => {
                let (lt, _) = self.operands(l, r, None, Need::Any);
                if op == BinOp::Eq && self.in_ensures && lt == Ty::Float {
                    self.push(Diag::new(
                        "W210",
                        e.pos,
                        "`==` on Float in `ensures`; use `approx`",
                    ));
                }
                Ty::Bool
            }
            BinOp::In => {
                if is_int_lit(l) {
                    let rt = self.expr(r, None);
                    let elem = match &rt {
                        Ty::Range(t) | Ty::List(t) => (**t).clone(),
                        Ty::Any => Ty::Any,
                        other => {
                            self.mismatch(r.pos, &Ty::List(Box::new(Ty::Any)), other);
                            Ty::Any
                        }
                    };
                    self.expr(l, Some(&elem));
                } else {
                    let lt = self.expr(l, None);
                    if let ExprKind::Range(a, b, _) = &r.kind {
                        if !lt.is_numeric() {
                            self.mismatch(l.pos, &Ty::Int, &lt);
                        }
                        self.expr(a, Some(&lt));
                        self.expr(b, Some(&lt));
                    } else {
                        let rt = self.expr(r, None);
                        match &rt {
                            Ty::List(t) | Ty::Range(t) => {
                                if unify(t, &lt).is_none() {
                                    self.mismatch(l.pos, t, &lt);
                                }
                            }
                            Ty::Any => {}
                            other => self.mismatch(r.pos, &Ty::List(Box::new(lt.clone())), other),
                        }
                    }
                }
                Ty::Bool
            }
        }
    }

    fn index(&mut self, e: &Expr, recv: &Expr, idx: &Expr) -> Ty {
        let rt = self.expr(recv, None);
        let int = Ty::Int;
        let slice = matches!(idx.kind, ExprKind::Range(..));
        let elem = match &rt {
            Ty::List(t) => Some((**t).clone()),
            Ty::Any => None,
            _ => {
                let found = self.src.get(recv.span.0..recv.span.1).unwrap_or("");
                let found = format!("{}[", found);
                self.push(
                    Diag::new("E204", e.pos, "this type cannot be indexed")
                        .with("found", found)
                        .with("on", rt.to_string())
                        .with("avail", prelude::avail(&rt, self.info)),
                );
                None
            }
        };
        if let ExprKind::Range(a, b, _) = &idx.kind {
            self.expr(a, Some(&int));
            self.expr(b, Some(&int));
        } else {
            self.expr(idx, Some(&int));
        }
        match (elem, slice) {
            (Some(t), true) => Ty::List(Box::new(t)),
            (Some(t), false) => t,
            (None, _) => Ty::Any,
        }
    }

    fn method(
        &mut self,
        e: &Expr,
        recv: &Expr,
        name: &str,
        name_end: usize,
        args: Option<&[Expr]>,
        exp: Option<&Ty>,
    ) -> Ty {
        let rt = self.expr(recv, None);
        let argv = args.unwrap_or(&[]);
        if rt == Ty::Any {
            for a in argv {
                self.expr(a, None);
            }
            return Ty::Any;
        }
        if let Ty::User(n) = &rt {
            if let Some(fields) = self.info.structs.get(n) {
                if let Some((_, fty)) = fields.iter().find(|(f, _)| f == name) {
                    if args.is_none() {
                        return fty.clone();
                    }
                }
            }
        }
        let avail = prelude::avail(&rt, self.info);
        if !avail.iter().any(|a| a == name)
            || (args.is_none() && !matches!(rt, Ty::User(_)))
            || (args.is_some() && matches!(rt, Ty::User(_)))
        {
            let found = self
                .src
                .get(recv.span.0..name_end)
                .unwrap_or(name)
                .to_string();
            let msg = if matches!(rt, Ty::User(_)) && avail.iter().any(|a| a == name) {
                "a field is not a method"
            } else if avail.iter().any(|a| a == name) {
                "method must be called with `()`"
            } else {
                "unknown member"
            };
            self.push(
                Diag::new("E204", e.pos, msg)
                    .with("found", found)
                    .with("on", rt.to_string())
                    .with("avail", avail),
            );
            for a in argv {
                self.expr(a, None);
            }
            return Ty::Any;
        }
        let want = prelude::lookup(&rt, name).map(|m| m.arity).unwrap_or(0);
        if argv.len() != want {
            self.arity_err(e.pos, want, argv.len());
        }
        let arg = |k: usize| argv.get(k);
        // range methods are typed as the `List<Int>` methods of the same name
        let rt = match rt {
            Ty::Range(t) => Ty::List(t),
            other => other,
        };
        match (&rt, name) {
            (Ty::List(_), "len") | (Ty::Str, "len") => Ty::Int,
            (Ty::List(t), "sum")
            | (Ty::List(t), "product")
            | (Ty::List(t), "min")
            | (Ty::List(t), "max")
                if **t == Ty::Any =>
            {
                match exp {
                    None => {
                        self.push(Diag::new(
                            "E313",
                            recv.pos,
                            "cannot infer the element type of `[]`: annotate with a `let`",
                        ));
                        Ty::Any
                    }
                    Some(Ty::Any) => Ty::Any,
                    Some(x) if !matches!(name, "sum" | "product") && !x.is_ordered() => {
                        let want = Ty::List(Box::new(x.clone()));
                        self.push(
                            Diag::new("E204", e.pos, "method needs an ordered element type")
                                .with("found", self.src.get(recv.span.0..name_end).unwrap_or(name))
                                .with("on", want.to_string())
                                .with("avail", prelude::avail(&want, self.info)),
                        );
                        Ty::Any
                    }
                    Some(x) => {
                        let numeric = matches!(name, "sum" | "product");
                        if numeric && *x == Ty::Float {
                            self.float_sums.insert(e.id);
                        }
                        if numeric && !matches!(x, Ty::Int | Ty::Float) {
                            Ty::Int
                        } else {
                            x.clone()
                        }
                    }
                }
            }
            (Ty::List(t), "sum") | (Ty::List(t), "product") => {
                if **t == Ty::Float {
                    self.float_sums.insert(e.id);
                }
                (**t).clone()
            }
            (Ty::List(t), "min") | (Ty::List(t), "max") => (**t).clone(),
            (Ty::List(t), "contains") | (Ty::List(t), "count") => {
                if let Some(a) = arg(0) {
                    self.expr(a, Some(t));
                }
                if name == "contains" {
                    Ty::Bool
                } else {
                    Ty::Int
                }
            }
            (Ty::List(_), "reverse") | (Ty::List(_), "sort") => rt.clone(),
            (Ty::List(t), "push") => {
                let mut out = rt.clone();
                if let Some(a) = arg(0) {
                    let at = self.expr(a, Some(t));
                    if t.contains_any() {
                        out = unify(&rt, &Ty::List(Box::new(at))).unwrap_or(rt.clone());
                    }
                }
                out
            }
            (Ty::List(t), "map") => {
                let mut out = Ty::Any;
                if let Some(a) = arg(0) {
                    let ft = Ty::Fn(vec![(**t).clone()], Box::new(Ty::Any));
                    if let Ty::Fn(_, r) = self.expr(a, Some(&ft)) {
                        out = *r;
                    }
                }
                Ty::List(Box::new(out))
            }
            (Ty::List(t), "filter") => {
                if let Some(a) = arg(0) {
                    let ft = Ty::Fn(vec![(**t).clone()], Box::new(Ty::Bool));
                    self.expr(a, Some(&ft));
                }
                rt.clone()
            }
            (Ty::List(t), "fold") => {
                let mut u = match arg(0) {
                    Some(a) => self.expr(a, None),
                    None => Ty::Any,
                };
                let Some(f) = arg(1) else {
                    return u;
                };
                // Re-check the lambda with the refined accumulator type until it is stable;
                // the intermediate passes must not leave diagnostics behind.
                let mut passes = 0;
                while u.contains_any() && passes < 4 {
                    passes += 1;
                    let (nd, ne) = (self.diags.len(), self.effect_calls);
                    let ft = Ty::Fn(vec![u.clone(), (**t).clone()], Box::new(u.clone()));
                    let res = self.expr(f, Some(&ft));
                    self.diags.truncate(nd);
                    self.effect_calls = ne;
                    let Ty::Fn(_, r) = res else {
                        break;
                    };
                    let next = unify(&u, &r).unwrap_or_else(|| u.clone());
                    if next == u {
                        break;
                    }
                    u = next;
                }
                let before = self.diags.len();
                let ft = Ty::Fn(vec![u.clone(), (**t).clone()], Box::new(u.clone()));
                let res = self.expr(f, Some(&ft));
                if self.diags.len() > before {
                    // the lambda already reported its errors: do not cascade
                    return Ty::Any;
                }
                match res {
                    Ty::Fn(_, r) => unify(&u, &r).unwrap_or(u),
                    _ => u,
                }
            }
            (Ty::List(_), "is_empty") | (Ty::Str, "is_empty") => Ty::Bool,
            (Ty::List(t), "any") | (Ty::List(t), "all") => {
                if let Some(a) = arg(0) {
                    let ft = Ty::Fn(vec![(**t).clone()], Box::new(Ty::Bool));
                    self.expr(a, Some(&ft));
                }
                Ty::Bool
            }
            (Ty::List(_), "take") | (Ty::List(_), "skip") => {
                if let Some(a) = arg(0) {
                    self.expr(a, Some(&Ty::Int));
                }
                rt.clone()
            }
            (Ty::List(t), "concat") => {
                let mut out = rt.clone();
                if let Some(a) = arg(0) {
                    let at = self.expr(a, Some(&rt));
                    if t.contains_any() {
                        out = unify(&rt, &at).unwrap_or(rt.clone());
                    }
                }
                out
            }
            (Ty::List(_), "dedup") => rt.clone(),
            (Ty::List(_), "join") => {
                if let Some(a) = arg(0) {
                    self.expr(a, Some(&Ty::Str));
                }
                Ty::Str
            }
            (Ty::Int, "min") | (Ty::Int, "max") | (Ty::Int, "pow") => {
                if let Some(a) = arg(0) {
                    self.expr(a, Some(&Ty::Int));
                }
                Ty::Int
            }
            (Ty::Float, "min") | (Ty::Float, "max") => {
                if let Some(a) = arg(0) {
                    self.expr(a, Some(&Ty::Float));
                }
                Ty::Float
            }
            (Ty::Float, "sqrt") | (Ty::Float, "floor") | (Ty::Float, "ceil") => Ty::Float,
            (Ty::Float, "round") => Ty::Float,
            (Ty::Float, "int") => Ty::Int,
            (_, "to_string") => Ty::Str,
            (Ty::Str, "chars") | (Ty::Str, "split") => {
                if let Some(a) = arg(0) {
                    self.expr(a, Some(&Ty::Str));
                }
                Ty::List(Box::new(Ty::Str))
            }
            (Ty::Str, "contains") | (Ty::Str, "starts_with") | (Ty::Str, "ends_with") => {
                if let Some(a) = arg(0) {
                    self.expr(a, Some(&Ty::Str));
                }
                Ty::Bool
            }
            (Ty::Str, "trim") | (Ty::Str, "to_uppercase") | (Ty::Str, "to_lowercase") => Ty::Str,
            (Ty::Str, "parse_int") => Ty::Result(Box::new(Ty::Int), Box::new(Ty::Str)),
            (Ty::Int, "float") | (Ty::Byte, "float") => Ty::Float,
            (Ty::Byte, "int") => Ty::Int,
            (Ty::Int, "abs") => Ty::Int,
            (Ty::Float, "abs") => Ty::Float,
            (Ty::Result(..), "is_ok") | (Ty::Result(..), "is_err") => Ty::Bool,
            (Ty::Result(t, _), "unwrap") => (**t).clone(),
            _ => Ty::Any,
        }
    }

    fn match_expr(&mut self, e: &Expr, s: &Expr, arms: &[Arm], exp: Option<&Ty>) -> Ty {
        let st = self.expr(s, None);
        let errs_before = self.diags.len();
        let mut pats: Vec<Pat> = Vec::new();
        let mut result: Option<Ty> = exp.cloned();
        for arm in arms {
            self.scopes.push(Vec::new());
            let p = self.pattern(&arm.pat, &st);
            pats.push(p);
            let bt = self.expr(&arm.body, result.as_ref());
            if result.is_none() {
                result = Some(bt);
            } else if let Some(r) = &result {
                if r.contains_any() {
                    result = unify(r, &bt).or(result);
                }
            }
            self.scopes.pop();
        }
        if self.diags.len() == errs_before && st != Ty::Any {
            let missing = exh::missing(self.info, &pats, &st);
            if !missing.is_empty() {
                self.push(
                    Diag::new("E310", e.pos, "non-exhaustive match").with("missing", missing),
                );
            }
        }
        result.unwrap_or(Ty::Any)
    }

    fn pattern(&mut self, p: &Pattern, st: &Ty) -> Pat {
        match &p.kind {
            PatKind::Wild => Pat::Wild,
            PatKind::Bind(n) => {
                self.declare(n, p.pos, st.clone());
                Pat::Wild
            }
            PatKind::Int(n) => {
                match st {
                    Ty::Byte => {
                        if !(0..=255).contains(n) {
                            self.push(
                                Diag::new("E302", p.pos, "literal out of range for Byte")
                                    .with("found", *n)
                                    .with("expected", "0..=255"),
                            );
                        }
                    }
                    Ty::Int | Ty::Any => {}
                    other => self.mismatch(p.pos, other, &Ty::Int),
                }
                Pat::Lit(format!("i{}", n))
            }
            PatKind::Float(x) => {
                if !matches!(st, Ty::Float | Ty::Any) {
                    self.mismatch(p.pos, st, &Ty::Float);
                }
                Pat::Lit(format!("f{}", x))
            }
            PatKind::Str(s) => {
                if !matches!(st, Ty::Str | Ty::Any) {
                    self.mismatch(p.pos, st, &Ty::Str);
                }
                Pat::Lit(format!("s{}", s))
            }
            PatKind::Bool(b) => {
                if !matches!(st, Ty::Bool | Ty::Any) {
                    self.mismatch(p.pos, st, &Ty::Bool);
                }
                Pat::Ctor(b.to_string(), vec![])
            }
            PatKind::Ctor(name, subs) => {
                let ctors = self.info.ctors(st);
                let payload: Option<Vec<Ty>> = match (&ctors, st) {
                    (Some(cs), _) => cs.iter().find(|(n, _)| n == name).map(|(_, t)| t.clone()),
                    (None, Ty::Any) => {
                        if name == "Ok" || name == "Err" {
                            Some(vec![Ty::Any])
                        } else {
                            self.info
                                .variants
                                .get(name)
                                .and_then(|en| self.info.enums.get(en))
                                .and_then(|vs| vs.iter().find(|(n, _)| n == name))
                                .map(|(_, t)| t.clone())
                        }
                    }
                    _ => None,
                };
                let Some(payload) = payload else {
                    let known =
                        name == "Ok" || name == "Err" || self.info.variants.contains_key(name);
                    if known || ctors.is_some() || !matches!(st, Ty::Any) {
                        if known {
                            let other = self
                                .info
                                .variants
                                .get(name)
                                .map(|n| Ty::User(n.clone()))
                                .unwrap_or_else(|| {
                                    Ty::Result(Box::new(Ty::Any), Box::new(Ty::Any))
                                });
                            self.mismatch(p.pos, st, &other);
                        } else {
                            self.push(
                                Diag::new("E101", p.pos, "unknown variant")
                                    .with("name", name.as_str()),
                            );
                        }
                    }
                    for sp in subs {
                        self.pattern(sp, &Ty::Any);
                    }
                    return Pat::Wild;
                };
                if payload.len() != subs.len() {
                    self.arity_err(p.pos, payload.len(), subs.len());
                }
                let mut out = Vec::new();
                for (k, sp) in subs.iter().enumerate() {
                    let pt = payload.get(k).cloned().unwrap_or(Ty::Any);
                    out.push(self.pattern(sp, &pt));
                }
                while out.len() < payload.len() {
                    out.push(Pat::Wild);
                }
                Pat::Ctor(name.clone(), out)
            }
        }
    }
}
