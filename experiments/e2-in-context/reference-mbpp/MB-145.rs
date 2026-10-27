```rust
fn max_abs_diff(arr: Vec<i64>) -> f64 {
    (arr.iter().max().unwrap() - arr.iter().min().unwrap()) as f64
}
```
