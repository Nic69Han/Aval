```rust
fn divisor(n: i64) -> i64 {
    (1..=n).filter(|i| n % i == 0).count() as i64
}
```
