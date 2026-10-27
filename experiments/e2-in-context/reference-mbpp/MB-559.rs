```rust
fn max_sub_array_sum(a: Vec<i64>, size: i64) -> i64 {
    let k = (size.max(0) as usize).min(a.len());
    let mut best = 0;
    let mut cur = 0;
    for &x in &a[..k] {
        cur = (cur + x).max(0);
        best = best.max(cur);
    }
    best
}
```
