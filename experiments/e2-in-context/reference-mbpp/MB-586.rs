```rust
fn split_arr(l: Vec<i64>, n: i64) -> Vec<i64> {
    let k = (n.max(0) as usize).min(l.len());
    let mut out = l[k..].to_vec();
    out.extend_from_slice(&l[..k]);
    out
}
```
