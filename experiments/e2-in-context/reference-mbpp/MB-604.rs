```rust
fn reverse_words(s: String) -> String {
    s.split(' ').filter(|w| !w.is_empty()).rev().collect::<Vec<_>>().join(" ")
}
```
