```rust
fn issort_list(list1: Vec<i64>) -> bool {
    list1.windows(2).all(|w| w[0] <= w[1])
}
```
