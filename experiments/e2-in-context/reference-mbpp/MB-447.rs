```rust
fn cube_nums(nums: Vec<f64>) -> Vec<f64> {
    nums.into_iter().map(|x| x * x * x).collect()
}
```
