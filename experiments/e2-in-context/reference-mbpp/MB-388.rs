```rust
fn highest_power_of_2(n: i64) -> i64 {
    let mut p = 1i64;
    while p * 2 <= n { p *= 2; }
    p
}
```
