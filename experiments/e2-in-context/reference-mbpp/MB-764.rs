```rust
fn number_ctr(s: String) -> i64 {
    s.chars().filter(|c| c.is_ascii_digit()).count() as i64
}
```
