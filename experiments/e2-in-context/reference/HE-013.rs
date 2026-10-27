```rust
fn greatest_common_divisor(a: i64, b: i64) -> i64 {
    if b == 0 {
        a
    } else {
        greatest_common_divisor(b, a % b)
    }
}
```
