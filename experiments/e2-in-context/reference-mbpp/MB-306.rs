```rust
fn max_sum_increasing_subseq(a: Vec<i64>, n: i64, index: i64, k: i64) -> i64 {
    let n = n as usize;
    let mut row: Vec<i64> = (0..n).map(|j| if a[j] > a[0] { a[j] + a[0] } else { a[j] }).collect();
    for i in 1..=(index.max(0) as usize) {
        let prev = row.clone();
        for j in 0..n {
            row[j] = if a[j] > a[i] && j > i {
                if prev[i] + a[j] > prev[j] { prev[i] + a[j] } else { prev[j] }
            } else {
                prev[j]
            };
        }
    }
    row[k as usize]
}
```
