```rust
fn sum_in_range(l: i64, r: i64) -> i64 {
    (l..=r).filter(|i| i % 2 != 0).sum()
}
```
