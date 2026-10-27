```rust
fn sum_even_and_even_index(arr: Vec<i64>) -> i64 {
    arr.iter().step_by(2).filter(|x| *x % 2 == 0).sum()
}
```
