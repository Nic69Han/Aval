```rust
fn how_many_times(string: String, substring: String) -> i64 {
    let chars: Vec<char> = string.chars().collect();
    let width = substring.chars().count();
    if width == 0 || width > chars.len() {
        return 0;
    }
    chars
        .windows(width)
        .filter(|w| w.iter().collect::<String>() == substring)
        .count() as i64
}
```
