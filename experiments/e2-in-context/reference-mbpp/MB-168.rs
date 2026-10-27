```rust
fn frequency(a: Vec<f64>, x: i64) -> i64 {
    a.iter().filter(|v| **v == x as f64).count() as i64
}
```
