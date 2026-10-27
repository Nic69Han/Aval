```rust
fn first_digit(n: i64) -> i64 {
    let mut m = n.abs();
    while m >= 10 {
        m /= 10;
    }
    m
}
```
