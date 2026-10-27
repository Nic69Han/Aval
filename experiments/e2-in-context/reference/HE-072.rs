```rust
fn will_it_fly(q: Vec<i64>, w: i64) -> bool {
    let mut reversed = q.clone();
    reversed.reverse();
    q == reversed && q.iter().sum::<i64>() <= w
}
```
