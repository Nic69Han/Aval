```rust
fn common(first: Vec<i64>, second: Vec<i64>) -> Vec<i64> {
    let mut result: Vec<i64> = first.into_iter().filter(|x| second.contains(x)).collect();
    result.sort();
    result.dedup();
    result
}
```
