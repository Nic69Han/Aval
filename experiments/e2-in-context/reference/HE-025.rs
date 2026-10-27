```rust
fn factorize(n: i64) -> Vec<i64> {
    let mut rest = n;
    let mut result = Vec::new();
    let mut d = 2;
    while d * d <= rest {
        while rest % d == 0 {
            result.push(d);
            rest /= d;
        }
        d += 1;
    }
    if rest > 1 {
        result.push(rest);
    }
    result
}
```
