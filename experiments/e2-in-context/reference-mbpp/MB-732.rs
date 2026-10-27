```rust
fn replace_specialchar(text: String) -> String {
    text.chars().map(|c| if c == ' ' || c == ',' || c == '.' { ':' } else { c }).collect()
}
```
