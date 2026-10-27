```rust
fn get_total_number_of_sequences(m: i64, n: i64) -> i64 {
    let (m, n) = (m as usize, n as usize);
    let mut t = vec![vec![0i64; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            t[i][j] = if i < j { 0 } else if j == 1 { i as i64 } else { t[i - 1][j] + t[i / 2][j - 1] };
        }
    }
    t[m][n]
}
```
