```rust
fn find_odd_pair(a: Vec<i64>, n: i64) -> i64 {
    let k = (n.max(0) as usize).min(a.len());
    let odd = a[..k].iter().filter(|x| *x % 2 != 0).count() as i64;
    odd * (k as i64 - odd)
}
```
