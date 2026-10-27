```rust
fn count_divisors(n: i64) -> bool {
    (1..=n).filter(|i| n % i == 0).count() % 2 == 0
}
```
