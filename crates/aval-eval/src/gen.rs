//! Seeded input generation, edge-case battery and shrinking (design D6, in-house).

use crate::value::Value;
use aval_check::{ModuleInfo, Ty};
use std::cmp::Ordering;
use std::rc::Rc;

/// splitmix64: tiny, fast, and fully deterministic.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    pub fn from_name(seed: u64, name: &str) -> Rng {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in name.bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        Rng(h ^ seed.wrapping_mul(0x9E37_79B9_7F4A_7C15))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n.max(1)
    }

    /// Inclusive range; only used with small spans.
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + self.below((hi - lo) as u64 + 1) as i64
    }

    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

pub struct Gen<'a> {
    pub info: &'a ModuleInfo,
}

fn rank(info: &ModuleInfo, ty: &Ty) -> Option<u32> {
    aval_check::info::rank_of(ty, &info.ranks)
}

impl<'a> Gen<'a> {
    pub fn new(info: &'a ModuleInfo) -> Gen<'a> {
        Gen { info }
    }

    fn variant_rank(&self, payload: &[Ty]) -> Option<u32> {
        let rs: Option<Vec<u32>> = payload.iter().map(|t| rank(self.info, t)).collect();
        rs.map(|r| 1 + r.into_iter().max().unwrap_or(0))
    }

    pub fn value(&self, ty: &Ty, rng: &mut Rng, depth: u32, nested: bool) -> Value {
        if depth > 4 {
            return self.minimal(ty);
        }
        match ty {
            Ty::Int => {
                let r = rng.below(100);
                if nested {
                    Value::Int(match r {
                        0..=4 => [i64::MAX, i64::MIN][rng.below(2) as usize],
                        5..=14 => [0, 1, -1][rng.below(3) as usize],
                        15..=74 => rng.range(-10, 10),
                        _ => rng.range(-1000, 1000),
                    })
                } else {
                    Value::Int(match r {
                        0..=7 => [0, 1, -1, i64::MAX, i64::MIN][rng.below(5) as usize],
                        8..=54 => rng.range(-10, 10),
                        55..=79 => rng.range(-1000, 1000),
                        80..=91 => rng.range(-1_000_000, 1_000_000),
                        _ => rng.next_u64() as i64,
                    })
                }
            }
            Ty::Byte => {
                if rng.below(100) < 20 {
                    Value::Byte([0, 1, 127, 128, 255][rng.below(5) as usize])
                } else {
                    Value::Byte(rng.below(256) as u8)
                }
            }
            Ty::Float => {
                let r = rng.below(100);
                let x = if nested && r < 3 {
                    [1e308, -1e308][rng.below(2) as usize]
                } else if r < 8 {
                    [0.0, -0.0, 1.0, -1.0, 0.5, -0.5, 2.0][rng.below(7) as usize]
                } else if r < 40 {
                    rng.range(-100, 100) as f64
                } else {
                    rng.unit() * 2000.0 - 1000.0
                };
                Value::Float(x)
            }
            Ty::Bool => Value::Bool(rng.below(2) == 1),
            Ty::Str => {
                const ALPHA: &[u8] = b"abcdxyz 019AZ_-";
                let len = rng.below(7);
                let s: String = (0..len)
                    .map(|_| ALPHA[rng.below(ALPHA.len() as u64) as usize] as char)
                    .collect();
                Value::str(&s)
            }
            Ty::List(t) => {
                let r = rng.below(100);
                let mut len = match r {
                    0..=14 => 0,
                    15..=34 => 1,
                    35..=59 => rng.range(2, 3),
                    60..=84 => rng.range(4, 8),
                    _ => rng.range(9, 32),
                } as usize;
                if depth >= 1 {
                    len = len.min(4);
                }
                Value::list(
                    (0..len)
                        .map(|_| self.value(t, rng, depth + 1, true))
                        .collect(),
                )
            }
            Ty::Result(t, e) => {
                let (rt, re) = (rank(self.info, t), rank(self.info, e));
                let ok = match (rt, re) {
                    (Some(_), None) => true,
                    (None, Some(_)) => false,
                    _ => rng.below(2) == 0,
                };
                if ok {
                    Value::ok(self.value(t, rng, depth + 1, true))
                } else {
                    Value::err(self.value(e, rng, depth + 1, true))
                }
            }
            Ty::User(n) => {
                if let Some(fields) = self.info.structs.get(n) {
                    let fs = fields
                        .iter()
                        .map(|(fname, fty)| (fname.clone(), self.value(fty, rng, depth + 1, true)))
                        .collect();
                    return Value::Struct(Rc::from(n.as_str()), Rc::new(fs));
                }
                if let Some(variants) = self.info.enums.get(n) {
                    let ranked: Vec<_> = variants
                        .iter()
                        .filter_map(|v| self.variant_rank(&v.1).map(|r| (v, r)))
                        .collect();
                    let least = ranked.iter().map(|(_, r)| *r).min();
                    let pool: Vec<_> = ranked
                        .iter()
                        .filter(|(_, r)| depth < 3 || Some(*r) == least)
                        .map(|(v, _)| *v)
                        .collect();
                    if pool.is_empty() {
                        return Value::Int(0);
                    }
                    let (vn, payload) = pool[rng.below(pool.len() as u64) as usize];
                    let vals = payload
                        .iter()
                        .map(|t| self.value(t, rng, depth + 1, true))
                        .collect();
                    return Value::variant(vn, vals);
                }
                Value::Int(0)
            }
            _ => Value::Int(0),
        }
    }

    /// The simplest value of a type (always terminates: it follows least construction ranks).
    pub fn minimal(&self, ty: &Ty) -> Value {
        match ty {
            Ty::Int => Value::Int(0),
            Ty::Byte => Value::Byte(0),
            Ty::Float => Value::Float(0.0),
            Ty::Bool => Value::Bool(false),
            Ty::Str => Value::str(""),
            Ty::List(_) => Value::list(vec![]),
            Ty::Result(t, e) => {
                let (rt, re) = (rank(self.info, t), rank(self.info, e));
                if rt.is_some() && (re.is_none() || rt <= re) {
                    Value::ok(self.minimal(t))
                } else {
                    Value::err(self.minimal(e))
                }
            }
            Ty::User(n) => {
                if let Some(fields) = self.info.structs.get(n) {
                    let fs = fields
                        .iter()
                        .map(|(f, t)| (f.clone(), self.minimal(t)))
                        .collect();
                    return Value::Struct(Rc::from(n.as_str()), Rc::new(fs));
                }
                if let Some(vs) = self.info.enums.get(n) {
                    let pick = vs
                        .iter()
                        .filter_map(|v| self.variant_rank(&v.1).map(|r| (v, r)))
                        .min_by_key(|(_, r)| *r);
                    if let Some(((vn, payload), _)) = pick {
                        let vals = payload.iter().map(|t| self.minimal(t)).collect();
                        return Value::variant(vn, vals);
                    }
                }
                Value::Int(0)
            }
            _ => Value::Int(0),
        }
    }

    /// Extreme scalar values that are also tried inside containers.
    fn extremes(&self, ty: &Ty) -> Vec<Value> {
        match ty {
            Ty::Int => vec![Value::Int(i64::MAX), Value::Int(i64::MIN)],
            Ty::Float => vec![Value::Float(1e308), Value::Float(-1e308)],
            Ty::Byte => vec![Value::Byte(255)],
            _ => vec![],
        }
    }

    /// Edge cases for a function parameter.
    pub fn edges(&self, ty: &Ty) -> Vec<Value> {
        match ty {
            Ty::Int => [0, 1, -1, i64::MAX, i64::MIN]
                .into_iter()
                .map(Value::Int)
                .collect(),
            Ty::Byte => [0, 1, 255, 128].into_iter().map(Value::Byte).collect(),
            Ty::Float => [0.0, -0.0, 1.0, -1.0, 1e308, -1e308]
                .into_iter()
                .map(Value::Float)
                .collect(),
            Ty::Bool => vec![Value::Bool(false), Value::Bool(true)],
            Ty::Str => ["", "a", "ab", " "].into_iter().map(Value::str).collect(),
            Ty::List(t) => {
                let safe = self.safe_edges(t);
                let mut out = vec![Value::list(vec![])];
                for s in safe.iter().take(3) {
                    out.push(Value::list(vec![s.clone()]));
                }
                for x in self.extremes(t) {
                    out.push(Value::list(vec![x]));
                }
                if let Some(s) = safe.first() {
                    out.push(Value::list(vec![s.clone(), s.clone(), s.clone()]));
                }
                let mut sorted = safe.clone();
                sorted.sort_by(crate::interp::interp_cmp);
                sorted.dedup();
                if sorted.len() > 1 {
                    out.push(Value::list(sorted.clone()));
                    sorted.reverse();
                    out.push(Value::list(sorted));
                }
                out
            }
            Ty::Result(t, e) => {
                let mut out = Vec::new();
                if rank(self.info, t).is_some() {
                    out.push(Value::ok(self.minimal(t)));
                    out.extend(self.extremes(t).into_iter().map(Value::ok));
                }
                if rank(self.info, e).is_some() {
                    out.push(Value::err(self.minimal(e)));
                    out.extend(self.extremes(e).into_iter().map(Value::err));
                }
                out
            }
            Ty::User(n) => {
                if let Some(fields) = self.info.structs.get(n) {
                    let mut out = vec![self.minimal(ty)];
                    for (k, (_, fty)) in fields.iter().enumerate() {
                        for x in self.extremes(fty) {
                            if let Value::Struct(name, fs) = self.minimal(ty) {
                                let mut nf = fs.to_vec();
                                nf[k].1 = x;
                                out.push(Value::Struct(name, Rc::new(nf)));
                            }
                        }
                    }
                    return out;
                }
                if let Some(vs) = self.info.enums.get(n) {
                    let mut out = Vec::new();
                    for (vn, p) in vs {
                        if self.variant_rank(p).is_none() {
                            continue;
                        }
                        let base: Vec<Value> = p.iter().map(|t| self.minimal(t)).collect();
                        out.push(Value::variant(vn, base.clone()));
                        for (k, t) in p.iter().enumerate() {
                            for x in self.extremes(t) {
                                let mut np = base.clone();
                                np[k] = x;
                                out.push(Value::variant(vn, np));
                            }
                        }
                    }
                    return out;
                }
                vec![]
            }
            _ => vec![Value::Int(0)],
        }
    }

    /// Small edge cases used for all-equal and sorted lists.
    pub fn safe_edges(&self, ty: &Ty) -> Vec<Value> {
        match ty {
            Ty::Int => [0, 1, -1].into_iter().map(Value::Int).collect(),
            Ty::Byte => [0, 1, 255].into_iter().map(Value::Byte).collect(),
            Ty::Float => [0.0, 1.0, -1.0].into_iter().map(Value::Float).collect(),
            Ty::Str => ["", "a"].into_iter().map(Value::str).collect(),
            other => self.edges(other).into_iter().take(3).collect(),
        }
    }

    /// Simpler candidates for a failing value, simplest first.
    pub fn shrink(&self, v: &Value, ty: &Ty) -> Vec<Value> {
        let mut out: Vec<Value> = Vec::new();
        match (v, ty) {
            (Value::Int(x), _) => out.extend(shrink_int(*x).into_iter().map(Value::Int)),
            (Value::Byte(x), _) => out.extend(
                shrink_int(i64::from(*x))
                    .into_iter()
                    .map(|i| Value::Byte(i as u8)),
            ),
            (Value::Float(x), _) => out.extend(shrink_float(*x).into_iter().map(Value::Float)),
            (Value::Bool(true), _) => out.push(Value::Bool(false)),
            (Value::Str(s), _) => {
                let cs: Vec<char> = s.chars().collect();
                if !cs.is_empty() {
                    out.push(Value::str(""));
                }
                for i in 0..cs.len() {
                    let mut c2 = cs.clone();
                    c2.remove(i);
                    out.push(Value::str(&c2.iter().collect::<String>()));
                }
                for i in 0..cs.len() {
                    if cs[i] != 'a' {
                        let mut c2 = cs.clone();
                        c2[i] = 'a';
                        out.push(Value::str(&c2.iter().collect::<String>()));
                    }
                }
            }
            (Value::List(xs), Ty::List(et)) => {
                let n = xs.len();
                if n > 0 {
                    out.push(Value::list(vec![]));
                }
                if n > 2 {
                    out.push(Value::list(xs[..n / 2].to_vec()));
                    out.push(Value::list(xs[n / 2..].to_vec()));
                }
                for i in 0..n {
                    let mut c = xs.to_vec();
                    c.remove(i);
                    out.push(Value::list(c));
                }
                for i in 0..n.saturating_sub(1) {
                    if let (Value::Int(a), Value::Int(b)) = (&xs[i], &xs[i + 1]) {
                        if let Some(s) = a.checked_add(*b) {
                            let mut c = xs.to_vec();
                            c[i] = Value::Int(s);
                            c.remove(i + 1);
                            out.push(Value::list(c));
                        }
                    }
                }
                for i in 0..n {
                    for cand in self.shrink(&xs[i], et) {
                        let mut c = xs.to_vec();
                        c[i] = cand;
                        out.push(Value::list(c));
                    }
                }
            }
            (Value::Variant(name, p), Ty::Result(t, e)) => {
                if &**name == "Err" {
                    out.push(Value::ok(self.minimal(t)));
                    for c in self.shrink(&p[0], e) {
                        out.push(Value::err(c));
                    }
                } else {
                    for c in self.shrink(&p[0], t) {
                        out.push(Value::ok(c));
                    }
                }
            }
            (Value::Variant(name, p), Ty::User(en)) => {
                if let Some(vs) = self.info.enums.get(en) {
                    let cur = vs.iter().find(|(n, _)| **n == **name);
                    let cur_rank = cur
                        .and_then(|(_, pt)| self.variant_rank(pt))
                        .unwrap_or(u32::MAX);
                    // a recursive payload of the same type replaces the whole value
                    for pv in p.iter() {
                        if matches!(pv, Value::Variant(..)) && pv.weight() < v.weight() {
                            if let Some((_, pt)) = cur {
                                if pt.iter().any(|t| *t == Ty::User(en.clone())) {
                                    out.push(pv.clone());
                                }
                            }
                        }
                    }
                    // variants of lower construction rank, least first
                    let mut lower: Vec<_> = vs
                        .iter()
                        .filter_map(|(vn, pt)| self.variant_rank(pt).map(|r| (r, vn, pt)))
                        .filter(|(r, _, _)| *r < cur_rank)
                        .collect();
                    lower.sort_by_key(|(r, _, _)| *r);
                    for (_, vn, pt) in lower {
                        let vals = pt.iter().map(|t| self.minimal(t)).collect();
                        out.push(Value::variant(vn, vals));
                    }
                    if let Some((_, pt)) = cur {
                        for (k, pv) in p.iter().enumerate() {
                            for c in self.shrink(pv, &pt[k]) {
                                let mut np = p.to_vec();
                                np[k] = c;
                                out.push(Value::variant(name, np));
                            }
                        }
                    }
                }
            }
            (Value::Struct(name, fields), Ty::User(sn)) => {
                if let Some(decl) = self.info.structs.get(sn) {
                    for (k, (_, fv)) in fields.iter().enumerate() {
                        for c in self.shrink(fv, &decl[k].1) {
                            let mut nf = fields.to_vec();
                            nf[k].1 = c;
                            out.push(Value::Struct(name.clone(), Rc::new(nf)));
                        }
                    }
                }
            }
            _ => {}
        }
        out.retain(|c| c != v);
        out
    }
}

/// Integers toward 0: 0 first, then values approaching `x` from below (binary-search style).
pub fn shrink_int(x: i64) -> Vec<i64> {
    if x == 0 {
        return vec![];
    }
    let neg = x < 0;
    let a = i128::from(x).abs();
    let mut mags: Vec<i128> = vec![0];
    let mut d = a / 2;
    while d >= 1 {
        mags.push(a - d);
        d /= 2;
    }
    mags.dedup();
    let mut out = Vec::new();
    if neg {
        if let Ok(p) = i64::try_from(a) {
            out.push(p);
        }
    }
    for m in mags {
        if m < a {
            let v = if neg { -m } else { m };
            if let Ok(v) = i64::try_from(v) {
                out.push(v);
            }
        }
    }
    // simplest first: smaller magnitude before larger
    out.sort_by_key(|v| (i128::from(*v).abs(), *v < 0));
    out.dedup();
    out
}

/// Floats toward 0.0, then toward integral values.
pub fn shrink_float(x: f64) -> Vec<f64> {
    if x == 0.0 && x.is_sign_positive() {
        return vec![];
    }
    if !x.is_finite() {
        return vec![0.0, 1.0];
    }
    let mut out = vec![0.0];
    if x.fract() != 0.0 {
        out.push(x.trunc());
        out.push(x.round());
        if x < 0.0 {
            out.push(-x);
        }
    } else if x.abs() < 9.0e15 {
        out.extend(shrink_int(x as i64).into_iter().map(|i| i as f64));
    } else {
        out.push(x / 2.0);
        out.push(x.trunc() / 4.0);
    }
    out.retain(|c| *c != x || (c.is_sign_negative() != x.is_sign_negative()));
    let mut seen: Vec<f64> = Vec::new();
    out.retain(|c| {
        if seen.iter().any(|s| s.to_bits() == c.to_bits()) {
            false
        } else {
            seen.push(*c);
            true
        }
    });
    out.sort_by(|a, b| a.abs().partial_cmp(&b.abs()).unwrap_or(Ordering::Equal));
    out
}

/// Greedy shrinking of a failing argument tuple. `still_fails` must also re-check `requires`.
/// The flag is `true` when the shrink budget ran out before a local minimum was reached.
pub fn shrink_args(
    gen: &Gen,
    args: Vec<Value>,
    tys: &[Ty],
    still_fails: &mut dyn FnMut(&[Value]) -> bool,
) -> (Vec<Value>, bool) {
    let mut cur = args;
    let mut budget = 4000;
    'outer: loop {
        for i in 0..cur.len() {
            for cand in gen.shrink(&cur[i], &tys[i]) {
                if budget == 0 {
                    return (cur, true);
                }
                budget -= 1;
                let mut t = cur.clone();
                t[i] = cand;
                if still_fails(&t) {
                    cur = t;
                    continue 'outer;
                }
            }
        }
        return (cur, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_is_deterministic() {
        let mut a = Rng::from_name(0, "f");
        let mut b = Rng::from_name(0, "f");
        assert_eq!(a.next_u64(), b.next_u64());
        assert_ne!(
            Rng::from_name(1, "f").next_u64(),
            Rng::from_name(0, "f").next_u64()
        );
        assert_ne!(
            Rng::from_name(0, "g").next_u64(),
            Rng::from_name(0, "f").next_u64()
        );
    }

    #[test]
    fn int_shrink_converges_to_threshold() {
        // smallest x >= 100 that "fails"
        let fails = |x: i64| x >= 100;
        let mut cur = 987_654_321_i64;
        while let Some(c) = shrink_int(cur).into_iter().find(|c| fails(*c)) {
            cur = c;
        }
        assert_eq!(cur, 100);
    }

    #[test]
    fn int_shrink_handles_extremes() {
        assert!(shrink_int(i64::MIN).contains(&0));
        assert!(shrink_int(i64::MAX).contains(&0));
        assert!(shrink_int(0).is_empty());
        assert_eq!(shrink_int(-5)[0], 0);
    }

    #[test]
    fn float_shrink_prefers_zero_then_integral() {
        assert_eq!(shrink_float(2.5)[0], 0.0);
        assert!(shrink_float(2.5).contains(&2.0));
        assert!(shrink_float(3.0).contains(&2.0));
        assert!(shrink_float(0.0).is_empty());
    }

    #[test]
    fn list_shrinks_by_removal_then_elements() {
        let info = ModuleInfo::default();
        let g = Gen::new(&info);
        let ty = Ty::List(Box::new(Ty::Int));
        let v = Value::list(vec![Value::Int(3), Value::Int(4)]);
        let c = g.shrink(&v, &ty);
        assert_eq!(c[0], Value::list(vec![]));
        assert!(c.contains(&Value::list(vec![Value::Int(7)])));
        assert!(c.contains(&Value::list(vec![Value::Int(4)])));
    }
}
