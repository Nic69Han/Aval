```rust
fn extract_string(str1: Vec<String>, l: i64) -> Vec<String> {
    str1.into_iter().filter(|s| s.len() as i64 == l).collect()
}
```
