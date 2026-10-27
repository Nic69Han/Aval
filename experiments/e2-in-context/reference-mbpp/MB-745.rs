```rust
fn divisible_by_digits(startnum: i64, endnum: i64) -> Vec<i64> {
    (startnum..=endnum)
        .filter(|&n| n > 0 && n.to_string().chars().all(|c| c != '0' && n % (c.to_digit(10).unwrap() as i64) == 0))
        .collect()
}
```
