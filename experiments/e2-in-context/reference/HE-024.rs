```rust
fn largest_divisor(n: i64) -> i64 {
    (1..n).rev().find(|d| n % d == 0).unwrap_or(1)
}
```
