```rust
fn first_odd(nums: Vec<i64>) -> i64 {
    nums.iter().cloned().find(|x| x % 2 != 0).unwrap_or(-1)
}
```
