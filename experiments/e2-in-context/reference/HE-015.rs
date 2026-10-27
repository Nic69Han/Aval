```rust
fn string_sequence(n: i64) -> String {
    (0..=n)
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}
```
