```rust
fn extract(lst: Vec<Vec<i64>>) -> Vec<i64> {
    lst.iter().map(|s| s[0]).collect()
}
```
