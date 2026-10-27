```rust
fn filter_oddnumbers(nums: Vec<i64>) -> Vec<i64> {
    nums.into_iter().filter(|x| x % 2 != 0).collect()
}
```
