```rust
fn count(lst: Vec<bool>) -> i64 {
    lst.iter().filter(|b| **b).count() as i64
}
```
