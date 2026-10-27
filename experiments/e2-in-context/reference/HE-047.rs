```rust
fn median(numbers: Vec<i64>) -> f64 {
    let mut s = numbers.clone();
    s.sort();
    let n = s.len();
    if n % 2 == 1 {
        s[n / 2] as f64
    } else {
        (s[n / 2 - 1] + s[n / 2]) as f64 / 2.0
    }
}
```
