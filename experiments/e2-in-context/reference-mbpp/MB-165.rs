```rust
fn count_char_position(str1: String) -> i64 {
    str1.chars().enumerate().filter(|(i, c)| c.to_ascii_lowercase() as i64 - 97 == *i as i64).count() as i64
}
```
