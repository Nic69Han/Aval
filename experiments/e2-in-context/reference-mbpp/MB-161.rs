```rust
fn remove_elements(list1: Vec<i64>, list2: Vec<i64>) -> Vec<i64> {
    list1.into_iter().filter(|x| !list2.contains(x)).collect()
}
```
