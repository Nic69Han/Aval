```rust
fn max_subarray_product(arr: Vec<i64>) -> i64 {
    let mut best = arr[0];
    let mut cmax = arr[0];
    let mut cmin = arr[0];
    for &x in &arr[1..] {
        let a = cmax * x;
        let b = cmin * x;
        cmax = x.max(a).max(b);
        cmin = x.min(a).min(b);
        best = best.max(cmax);
    }
    best
}
```
