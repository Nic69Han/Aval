```rust
fn monotonic(numbers: Vec<i64>) -> bool {
    let up = numbers.windows(2).all(|w| w[0] <= w[1]);
    let down = numbers.windows(2).all(|w| w[0] >= w[1]);
    up || down
}
```
