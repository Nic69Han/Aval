```rust
fn perfect_squares(a: i64, b: i64) -> Vec<i64> {
    let (lo, hi) = (a.min(b), a.max(b));
    (0..)
        .map(|i| i * i)
        .take_while(|&x| x <= hi)
        .filter(|&x| x >= lo)
        .collect()
}
```
