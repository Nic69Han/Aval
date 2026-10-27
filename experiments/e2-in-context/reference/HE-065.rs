```rust
fn circular_shift(x: i64, shift: i64) -> String {
    let digits: Vec<char> = x.to_string().chars().collect();
    let n = digits.len() as i64;
    if shift > n {
        digits.iter().rev().collect()
    } else {
        let cut = (n - shift) as usize;
        let mut out: String = digits[cut..].iter().collect();
        out.push_str(&digits[..cut].iter().collect::<String>());
        out
    }
}
```
