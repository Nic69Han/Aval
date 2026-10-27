```rust
fn odd_equivalent(s: String, n: i64) -> i64 {
    s.chars().take(n.max(0) as usize).filter(|c| *c == '1').count() as i64
}
```
