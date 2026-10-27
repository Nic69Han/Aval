```rust
fn maximum(arr: Vec<i64>, k: i64) -> Vec<i64> {
    let mut s = arr.clone();
    s.sort();
    let start = s.len() - k as usize;
    s[start..].to_vec()
}
```
