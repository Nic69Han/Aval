```rust
fn toggle_string(string: String) -> String {
    string
        .chars()
        .map(|c| if c.is_uppercase() { c.to_lowercase().to_string() } else { c.to_uppercase().to_string() })
        .collect()
}
```
