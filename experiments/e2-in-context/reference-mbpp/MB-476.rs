```rust
fn big_sum(nums: Vec<i64>) -> i64 {
    nums.iter().max().unwrap() + nums.iter().min().unwrap()
}
```
