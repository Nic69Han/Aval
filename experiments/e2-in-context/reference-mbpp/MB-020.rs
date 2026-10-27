```rust
fn is_woodall(x: f64) -> bool {
    if !(x >= 1.0 && x < 1e12 && x == x.floor()) { return false; }
    let y = x as i64;
    (1..=40i64).any(|k| k * 2i64.pow(k as u32) == y + 1)
}
```
