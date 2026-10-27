```rust
fn is_simple_power(x: i64, n: i64) -> bool {
    if x == 1 {
        true
    } else if n < 2 || x % n != 0 {
        false
    } else {
        is_simple_power(x / n, n)
    }
}
```
