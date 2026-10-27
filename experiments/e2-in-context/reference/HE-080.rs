```rust
fn is_happy(s: String) -> bool {
    let cs: Vec<char> = s.chars().collect();
    if cs.len() < 3 {
        return false;
    }
    cs.windows(3)
        .all(|w| w[0] != w[1] && w[0] != w[2] && w[1] != w[2])
}
```
