```rust
fn split(l: Vec<i64>) -> Vec<i64> {
    l.into_iter().filter(|x| x % 2 == 0).collect()
}
```
