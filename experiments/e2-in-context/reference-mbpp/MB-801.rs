```rust
fn test_three_equal(x: i64, y: i64, z: i64) -> i64 {
    if x == y && y == z {
        3
    } else if x == y || y == z || x == z {
        2
    } else {
        0
    }
}
```
