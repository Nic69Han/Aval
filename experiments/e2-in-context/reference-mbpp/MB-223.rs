```rust
fn is_majority(arr: Vec<i64>, n: i64, x: i64) -> bool {
    if !arr.contains(&x) { return false; }
    arr.iter().filter(|v| **v == x).count() as f64 > n as f64 / 2.0
}
```
