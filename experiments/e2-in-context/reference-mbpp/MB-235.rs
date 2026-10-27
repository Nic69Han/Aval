```rust
fn even_bit_set_number(n: i64) -> i64 {
    let mut n = n;
    let mut mask = 2i64;
    while mask < n {
        if (n / mask) % 2 == 0 { n += mask; }
        mask *= 4;
    }
    n
}
```
