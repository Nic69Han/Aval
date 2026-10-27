```rust
fn left_rotate(n: i64, d: i64) -> i64 {
    n * 2i64.pow(d as u32) + n / 2i64.pow((32 - d) as u32)
}
```
