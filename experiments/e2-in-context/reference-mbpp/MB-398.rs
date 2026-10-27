```rust
fn sum_of_digits(nums: Vec<i64>) -> i64 {
    nums.iter().map(|x| x.abs().to_string().chars().map(|c| c.to_digit(10).unwrap() as i64).sum::<i64>()).sum()
}
```
