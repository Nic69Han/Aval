```rust
fn two_unique_nums(nums: Vec<i64>) -> Vec<i64> {
    nums.iter().cloned().filter(|x| nums.iter().filter(|y| *y == x).count() == 1).collect()
}
```
