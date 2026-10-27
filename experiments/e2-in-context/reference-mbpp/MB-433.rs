```rust
fn check_greater(arr: Vec<f64>, number: i64) -> bool {
    arr.iter().all(|&v| v < number as f64)
}
```
