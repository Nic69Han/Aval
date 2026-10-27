```rust
fn sum_range_list(list1: Vec<i64>, m: i64, n: i64) -> i64 {
    list1.iter().skip(m as usize).take((n + 1 - m).max(0) as usize).sum()
}
```
