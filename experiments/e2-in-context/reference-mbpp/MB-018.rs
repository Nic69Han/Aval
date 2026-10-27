```rust
fn remove_dirty_chars(string: String, second_string: String) -> String {
    string.chars().filter(|c| !second_string.contains(*c)).collect()
}
```
