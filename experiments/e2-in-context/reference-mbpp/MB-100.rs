```rust
fn next_smallest_palindrome(num: i64) -> i64 {
    let pal = |k: i64| { let s = k.to_string(); s.chars().eq(s.chars().rev()) };
    let mut k = num + 1;
    while !pal(k) { k += 1; }
    k
}
```
