```rust
fn second_smallest(numbers: Vec<f64>) -> i64 {
    let mut v = numbers;
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v.dedup();
    v[1] as i64
}
```
