```rust
fn sequence(n: i64) -> i64 {
    if n < 3 { return 1; }
    let n = n as usize;
    let mut s = vec![0usize; n + 1];
    s[1] = 1;
    s[2] = 1;
    for i in 3..=n {
        s[i] = s[s[i - 1]] + s[i - s[i - 1]];
    }
    s[n] as i64
}
```
