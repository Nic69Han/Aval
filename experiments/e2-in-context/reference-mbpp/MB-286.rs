```rust
fn max_sub_array_sum_repeated(a: Vec<i64>, n: i64, k: i64) -> i64 {
    let mut modified: Vec<i64> = Vec::new();
    for _ in 0..k { modified.extend(a.iter()); }
    let mut pre = 0i64;
    let mut res = modified[0];
    for x in modified {
        pre = (pre + x).max(x);
        res = res.max(pre);
    }
    res
}
```
