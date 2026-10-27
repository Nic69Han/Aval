```rust
fn is_polite(n: i64) -> i64 {
    let mut m = n + 1;
    loop {
        if m - (64 - (m as u64).leading_zeros() as i64) == n {
            return m;
        }
        m += 1;
    }
}
```
