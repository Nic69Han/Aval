```rust
fn is_not_prime(n: i64) -> bool {
    if n == 1 { return true; }
    let mut d = 2;
    while d * d <= n {
        if n % d == 0 { return true; }
        d += 1;
    }
    false
}
```
