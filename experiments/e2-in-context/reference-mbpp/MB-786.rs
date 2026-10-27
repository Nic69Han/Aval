```rust
fn right_insertion(a: Vec<f64>, x: i64) -> i64 {
    let (mut lo, mut hi) = (0usize, a.len());
    while lo < hi {
        let mid = (lo + hi) / 2;
        if (x as f64) < a[mid] {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo as i64
}
```
