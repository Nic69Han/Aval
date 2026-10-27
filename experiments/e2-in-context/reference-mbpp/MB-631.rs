```rust
fn replace_spaces(text: String) -> String {
    text.chars()
        .map(|c| match c {
            ' ' => '_',
            '_' => ' ',
            o => o,
        })
        .collect()
}
```
