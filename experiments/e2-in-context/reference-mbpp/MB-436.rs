```rust
fn neg_nos(list1: Vec<i64>) -> Vec<i64> {
    list1.into_iter().filter(|&x| x < 0).collect()
}
```
