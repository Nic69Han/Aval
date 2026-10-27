```rust
fn long_words(n: i64, s: String) -> Vec<String> {
    s.split(' ').filter(|w| w.len() as i64 > n).map(|w| w.to_string()).collect()
}
```
