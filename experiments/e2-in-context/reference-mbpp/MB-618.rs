```rust
fn div_list(nums1: Vec<i64>, nums2: Vec<i64>) -> Vec<f64> {
    nums1.iter().zip(nums2.iter()).map(|(&a, &b)| a as f64 / b as f64).collect()
}
```
