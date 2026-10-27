```rust
fn validate(n: i64) -> bool {
    let ds: Vec<char> = n.to_string().chars().collect();
    ds.iter().all(|d| d.to_digit(10).unwrap() as usize >= ds.iter().filter(|e| *e == d).count())
}
```
