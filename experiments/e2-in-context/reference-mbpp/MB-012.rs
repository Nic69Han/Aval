```rust
fn sort_matrix(m: Vec<Vec<i64>>) -> Vec<Vec<i64>> {
    let mut r = m.clone();
    r.sort_by_key(|row| row.iter().sum::<i64>());
    r
}
```
