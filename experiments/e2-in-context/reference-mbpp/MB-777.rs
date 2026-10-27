```rust
fn find_sum(arr: Vec<i64>) -> i64 {
    let mut v = arr;
    v.sort();
    v.dedup();
    v.iter().sum()
}
```
