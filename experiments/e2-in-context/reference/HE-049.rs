```rust
fn modp(n: i64, p: i64) -> i64 {
    let mut acc: i64 = 1;
    for _ in 0..n {
        acc = acc * 2 % p;
    }
    acc
}
```
