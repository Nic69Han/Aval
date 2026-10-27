```rust
fn split(word: String) -> Vec<String> {
    word.chars().map(|c| c.to_string()).collect()
}
```
