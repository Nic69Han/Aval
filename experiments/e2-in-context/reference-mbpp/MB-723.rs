```rust
fn count_same_pair(nums1: Vec<i64>, nums2: Vec<i64>) -> i64 {
    nums1.iter().zip(nums2.iter()).filter(|(a, b)| a == b).count() as i64
}
```
