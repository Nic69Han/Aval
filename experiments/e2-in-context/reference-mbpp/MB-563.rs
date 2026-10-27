```rust
fn extract_values(text: String) -> Vec<String> {
    let parts: Vec<&str> = text.split('"').collect();
    let n = parts.len();
    (0..n).filter(|i| i % 2 == 1 && i + 1 < n).map(|i| parts[i].to_string()).collect()
}
```
