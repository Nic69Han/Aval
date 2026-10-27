```rust
fn smallest_num(xs: Vec<f64>) -> f64 {
    xs.iter().cloned().fold(f64::INFINITY, f64::min)
}
```
