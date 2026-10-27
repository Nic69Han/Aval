```rust
fn even_power_sum(n: i64) -> i64 {
    (1..=n).map(|k| (2 * k).pow(5)).sum()
}
```
