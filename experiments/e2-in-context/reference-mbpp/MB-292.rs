```rust
fn find(n: i64, m: i64) -> i64 {
    n.div_euclid(m) - if m < 0 && n.rem_euclid(m) != 0 { 1 } else { 0 }
}
```
