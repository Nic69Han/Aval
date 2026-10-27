```rust
fn all_unique(test_list: Vec<i64>) -> bool {
    let mut s = test_list.clone();
    s.sort();
    s.dedup();
    s.len() == test_list.len()
}
```
