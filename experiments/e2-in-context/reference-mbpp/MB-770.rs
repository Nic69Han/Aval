```rust
fn odd_num_sum(n: i64) -> i64 {
    (1..=n).map(|i| (2 * i - 1).pow(4)).sum()
}
```
