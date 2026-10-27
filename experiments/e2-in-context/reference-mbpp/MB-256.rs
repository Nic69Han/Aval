```rust
fn count_primes_nums(n: i64) -> i64 {
    (2..n).filter(|&i| (2..).take_while(|d| d * d <= i).all(|d| i % d != 0)).count() as i64
}
```
