```rust
fn count_binary_seq(n: i64) -> f64 {
    let mut ncr = 1.0f64;
    let mut res = 1.0f64;
    for r in 1..=n {
        ncr = ncr * (n + 1 - r) as f64 / r as f64;
        res += ncr * ncr;
    }
    res
}
```
