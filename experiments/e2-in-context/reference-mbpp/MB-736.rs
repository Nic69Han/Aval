```rust
fn left_insertion(a: Vec<i64>, x: i64) -> i64 {
    let (mut lo, mut hi) = (0usize, a.len());
    while lo < hi {
        let mid = (lo + hi) / 2;
        if a[mid] < x {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo as i64
}
```
