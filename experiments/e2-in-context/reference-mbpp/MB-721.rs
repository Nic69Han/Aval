```rust
fn max_average_of_path(cost_: Vec<Vec<i64>>) -> f64 {
    let n = cost_.len();
    let mut dp = vec![vec![0i64; n]; n];
    for i in 0..n {
        for j in 0..n {
            let best = if i == 0 && j == 0 {
                0
            } else if i == 0 {
                dp[i][j - 1]
            } else if j == 0 {
                dp[i - 1][j]
            } else {
                dp[i - 1][j].max(dp[i][j - 1])
            };
            dp[i][j] = cost_[i][j] + best;
        }
    }
    dp[n - 1][n - 1] as f64 / (2 * n - 1) as f64
}
```
