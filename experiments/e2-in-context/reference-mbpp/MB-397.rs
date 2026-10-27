```rust
fn median_numbers(a: i64, b: i64, c: i64) -> i64 {
    a.min(b).max(a.max(b).min(c))
}
```
