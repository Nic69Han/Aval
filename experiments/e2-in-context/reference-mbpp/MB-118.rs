```rust
fn string_to_list(string: String) -> Vec<String> {
    string.split(' ').map(|s| s.to_string()).collect()
}
```
