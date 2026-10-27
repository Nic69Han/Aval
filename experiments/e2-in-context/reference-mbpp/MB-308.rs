```rust
fn large_product(nums1: Vec<f64>, nums2: Vec<f64>, n: i64) -> Vec<f64> {
    let mut products: Vec<f64> = Vec::new();
    for x in &nums1 { for y in &nums2 { products.push(x * y); } }
    products.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let take = if n >= 0 { n as usize } else { (products.len() as i64 + n).max(0) as usize };
    products.into_iter().take(take).collect()
}
```
