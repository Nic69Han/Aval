```rust
fn overlapping(list1: Vec<i64>, list2: Vec<i64>) -> bool {
    list1.iter().any(|v| list2.contains(v))
}
```
