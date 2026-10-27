```rust
fn sub_list(nums1: Vec<f64>, nums2: Vec<f64>) -> Vec<f64> {
    nums1.iter().zip(nums2.iter()).map(|(a, b)| a - b).collect()
}
```
