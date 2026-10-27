```rust
fn find_lucas(n: i64) -> i64 {
    let (mut a, mut b) = (2i64, 1i64);
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}
```
