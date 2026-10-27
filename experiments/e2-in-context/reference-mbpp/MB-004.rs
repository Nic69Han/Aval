```rust
fn heap_queue_largest(nums: Vec<i64>, n: i64) -> Vec<i64> {
    let mut s = nums.clone();
    s.sort();
    s.reverse();
    s.into_iter().take(n.max(0) as usize).collect()
}
```
