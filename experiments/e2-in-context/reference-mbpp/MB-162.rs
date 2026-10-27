```rust
fn sum_series(n: i64) -> i64 {
    if n <= 0 { return 0; }
    (0..=n / 2).map(|i| n - 2 * i).sum()
}
```
