```rust
fn is_decimal(num: String) -> bool {
    let d = |s: &str| s.chars().all(|c| c.is_ascii_digit());
    let parts: Vec<&str> = num.split('.').collect();
    match parts.len() {
        1 => !parts[0].is_empty() && d(parts[0]),
        2 => !parts[0].is_empty() && d(parts[0]) && !parts[1].is_empty() && parts[1].len() <= 2 && d(parts[1]),
        _ => false,
    }
}
```
