```rust
fn extract_quotation(text1: String) -> Vec<String> {
    let parts: Vec<&str> = text1.split('"').collect();
    let n = parts.len();
    (0..n).filter(|i| i % 2 == 1 && i + 1 < n).map(|i| parts[i].to_string()).collect()
}
```
