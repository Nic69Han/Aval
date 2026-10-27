```rust
fn len_log(list1: Vec<String>) -> i64 {
    list1.iter().map(|w| w.len() as i64).max().unwrap()
}
```
