```rust
fn next_perfect_square(n: f64) -> i64 {
    if n < 0.0 { return 0; }
    let k = n.sqrt().floor() as i64 + 1;
    k * k
}
```
