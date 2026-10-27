```rust
fn vowels_count(s: String) -> i64 {
    let lower = s.to_lowercase();
    let plain = lower.chars().filter(|c| "aeiou".contains(*c)).count() as i64;
    if lower.ends_with('y') {
        plain + 1
    } else {
        plain
    }
}
```
