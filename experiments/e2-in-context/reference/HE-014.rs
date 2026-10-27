```rust
fn all_prefixes(string: String) -> Vec<String> {
    let chars: Vec<char> = string.chars().collect();
    (1..=chars.len())
        .map(|i| chars[..i].iter().collect::<String>())
        .collect()
}
```
