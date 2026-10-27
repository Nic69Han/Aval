```rust
fn find_even_pair(a: Vec<i64>) -> i64 {
    let evens = a.iter().filter(|x| *x % 2 == 0).count() as i64;
    let odds = a.len() as i64 - evens;
    evens * (evens - 1) / 2 + odds * (odds - 1) / 2
}
```
