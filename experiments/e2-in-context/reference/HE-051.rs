```rust
fn remove_vowels(text: String) -> String {
    text.chars().filter(|c| !"aeiouAEIOU".contains(*c)).collect()
}
```
