```rust
fn find_max_num(arr: Vec<i64>) -> i64 {
    let mut s = arr.clone();
    s.sort();
    s.reverse();
    s.iter().fold(0, |acc, d| acc * 10 + d)
}
```
