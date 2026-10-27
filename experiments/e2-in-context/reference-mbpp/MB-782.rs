```rust
fn odd_length_sum(arr: Vec<i64>) -> i64 {
    let n = arr.len() as i64;
    arr.iter().enumerate().map(|(i, &v)| v * (((i as i64 + 1) * (n - i as i64) + 1) / 2)).sum()
}
```
