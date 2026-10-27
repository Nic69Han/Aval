```rust
fn test_duplicate(arraynums: Vec<i64>) -> bool {
    let mut s = arraynums.clone();
    s.sort();
    s.dedup();
    s.len() < arraynums.len()
}
```
