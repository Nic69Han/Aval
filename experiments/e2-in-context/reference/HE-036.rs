```rust
fn fizz_buzz(n: i64) -> i64 {
    let mut count = 0;
    for i in 0..n {
        if i % 11 == 0 || i % 13 == 0 {
            count += i.to_string().chars().filter(|c| *c == '7').count() as i64;
        }
    }
    count
}
```
