```rust
fn power(a: i64, b: i64) -> f64 {
    let mut p = 1.0f64;
    for _ in 0..b.abs() { p *= a as f64; }
    if b >= 0 { p } else { 1.0 / p }
}
```
