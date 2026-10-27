```rust
fn next_power_of_2(n: i64) -> i64 {
    let mut p = 1i64;
    while p < n { p *= 2; }
    p
}
```
