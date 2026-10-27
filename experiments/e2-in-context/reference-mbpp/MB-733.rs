```rust
fn find_first_occurrence(a: Vec<i64>, x: i64) -> i64 {
    a.iter().position(|&v| v == x).map_or(-1, |i| i as i64)
}
```
