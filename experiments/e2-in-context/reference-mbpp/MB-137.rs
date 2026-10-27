```rust
fn zero_count(nums: Vec<i64>) -> f64 {
    let zeros = nums.iter().filter(|x| **x == 0).count() as f64;
    zeros / (nums.len() as f64 - zeros)
}
```
