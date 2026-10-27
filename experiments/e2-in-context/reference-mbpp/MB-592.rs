```rust
fn sum_of_product(n: i64) -> i64 {
    let mut c = 1;
    for k in 0..(n - 1).max(0) {
        c = c * (2 * n - k) / (k + 1);
    }
    c
}
```
