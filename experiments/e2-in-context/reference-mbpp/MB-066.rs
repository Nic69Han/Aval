```rust
fn pos_count(l: Vec<f64>) -> i64 {
    l.iter().filter(|x| **x > 0.0).count() as i64
}
```
