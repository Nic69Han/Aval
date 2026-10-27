```rust
fn rotate_right(l: Vec<i64>, m: i64) -> Vec<i64> {
    let n = l.len() as i64;
    if m < 0 {
        let k = ((-m).min(n)) as usize;
        let mut out = l[k..].to_vec();
        out.extend_from_slice(&l[..k]);
        out
    } else {
        let cut = (n - m).max(0) as usize;
        let mut out = l[cut..].to_vec();
        out.extend_from_slice(&l[..cut]);
        out
    }
}
```
