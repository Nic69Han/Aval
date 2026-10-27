```rust
fn sum_negativenum(nums: Vec<f64>) -> f64 {
    nums.iter().filter(|x| **x < 0.0).sum()
}
```
