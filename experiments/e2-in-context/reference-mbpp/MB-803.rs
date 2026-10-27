```rust
fn is_perfect_square(n: i64) -> bool {
    if n < 0 {
        return false;
    }
    let r = (n as f64).sqrt() as i64;
    r * r == n || (r + 1) * (r + 1) == n
}
```
