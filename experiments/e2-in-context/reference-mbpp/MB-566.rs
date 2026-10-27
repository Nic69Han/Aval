```rust
fn sum_digits(n: i64) -> i64 {
    let mut m = n;
    let mut s = 0;
    while m > 0 {
        s += m % 10;
        m /= 10;
    }
    s
}
```
