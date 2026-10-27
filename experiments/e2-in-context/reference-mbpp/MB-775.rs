```rust
fn odd_position(nums: Vec<i64>) -> bool {
    nums.iter().enumerate().all(|(i, &v)| i % 2 == 0 || v % 2 != 0)
}
```
