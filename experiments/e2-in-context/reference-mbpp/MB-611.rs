```rust
fn max_of_nth(test_list: Vec<Vec<i64>>, n: i64) -> i64 {
    test_list
        .iter()
        .map(|r| if n < 0 { r[(r.len() as i64 + n) as usize] } else { r[n as usize] })
        .max()
        .unwrap()
}
```
