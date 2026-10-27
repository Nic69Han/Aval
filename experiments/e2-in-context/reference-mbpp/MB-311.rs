```rust
fn set_left_most_unset_bit(n: i64) -> i64 {
    if n <= 0 { return n; }
    let mut best: i64 = -1;
    let mut t = n;
    let mut pos = 0;
    while t > 0 {
        if t % 2 == 0 { best = pos; }
        pos += 1;
        t /= 2;
    }
    if best < 0 { n } else { n + (1i64 << best) }
}
```
