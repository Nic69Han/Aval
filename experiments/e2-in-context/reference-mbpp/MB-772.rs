```rust
fn remove_length(test_str: String, k: i64) -> String {
    test_str
        .split(' ')
        .filter(|w| !w.is_empty() && w.len() as i64 != k)
        .collect::<Vec<_>>()
        .join(" ")
}
```
