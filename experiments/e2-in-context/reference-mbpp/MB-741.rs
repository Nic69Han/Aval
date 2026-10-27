```rust
fn all_characters_same(s: String) -> bool {
    let mut cs = s.chars();
    match cs.next() {
        Some(f) => cs.all(|c| c == f),
        None => true,
    }
}
```
