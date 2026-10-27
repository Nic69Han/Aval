```rust
fn count_set_bits(n: i64) -> i64 {
    n.abs().count_ones() as i64
}
```
