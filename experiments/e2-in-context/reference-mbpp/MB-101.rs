```rust
fn kth_element(arr: Vec<i64>, k: i64) -> i64 {
    if k >= 1 { arr[(k - 1) as usize] } else { arr[(arr.len() as i64 + k - 1) as usize] }
}
```
