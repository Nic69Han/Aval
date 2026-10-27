```rust
fn sum(a: i64, b: i64) -> f64 {
    fn gcd(a: i64, b: i64) -> i64 { if b == 0 { a.abs() } else { gcd(b, a % b) } }
    let g = gcd(a, b);
    let mut total = 0i64;
    let mut i = 1;
    while i * i <= g {
        if g % i == 0 {
            total += i;
            if g / i != i { total += g / i; }
        }
        i += 1;
    }
    total as f64
}
```
