```rust
fn sumof_factors(n: i64) -> i64 {
    (1..=n).filter(|i| n % i == 0 && i % 2 == 0).sum()
}
```
