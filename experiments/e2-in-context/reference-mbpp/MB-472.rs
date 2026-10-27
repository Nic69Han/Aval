```rust
fn check_consecutive(l: Vec<i64>) -> bool {
    let mut s = l.clone();
    s.sort();
    let lo = s[0];
    s.iter().enumerate().all(|(i, &v)| v == lo + i as i64)
}
```
