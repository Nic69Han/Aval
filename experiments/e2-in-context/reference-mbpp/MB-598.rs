```rust
fn armstrong_number(number: i64) -> bool {
    if number < 0 {
        return false;
    }
    let s = number.to_string();
    let p = s.len() as u32;
    s.chars().map(|c| (c.to_digit(10).unwrap() as i64).pow(p)).sum::<i64>() == number
}
```
