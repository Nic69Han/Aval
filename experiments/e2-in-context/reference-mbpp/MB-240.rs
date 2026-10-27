```rust
fn replace_list(list1: Vec<i64>, list2: Vec<i64>) -> Vec<i64> {
    let mut r: Vec<i64> = list1.iter().take(list1.len().saturating_sub(1)).cloned().collect();
    r.extend(list2);
    r
}
```
