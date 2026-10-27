```rust
fn nth_nums(nums: Vec<f64>, n: i64) -> Vec<f64> {
    nums.into_iter()
        .map(|x| {
            let p = (0..n.abs()).fold(1.0, |acc, _| acc * x);
            if n < 0 { 1.0 / p } else { p }
        })
        .collect()
}
```
