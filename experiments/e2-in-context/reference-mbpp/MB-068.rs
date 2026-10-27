```rust
fn is_monotonic(a: Vec<f64>) -> bool {
    a.windows(2).all(|w| w[0] <= w[1]) || a.windows(2).all(|w| w[0] >= w[1])
}
```
