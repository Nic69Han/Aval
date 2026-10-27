```rust
fn find_min_length(lst: Vec<Vec<i64>>) -> i64 {
    lst.iter().map(|l| l.len() as i64).min().unwrap()
}
```
