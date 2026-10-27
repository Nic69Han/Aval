```rust
fn newman_prime(n: i64) -> i64 {
    if n < 2 { return 1; }
    let (mut a, mut b) = (1i64, 1i64);
    for _ in 2..=n {
        let c = 2 * b + a;
        a = b;
        b = c;
    }
    b
}
```
