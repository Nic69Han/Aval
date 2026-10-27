```rust
fn derivative(xs: Vec<i64>) -> Vec<i64> {
    (1..xs.len()).map(|i| xs[i] * i as i64).collect()
}
```
