```rust
fn sort_numeric_strings(nums_str: Vec<String>) -> Vec<i64> {
    let mut v: Vec<i64> = nums_str.iter().map(|s| s.trim().parse().unwrap()).collect();
    v.sort();
    v
}
```
