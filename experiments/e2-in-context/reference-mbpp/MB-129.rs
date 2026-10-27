```rust
fn magic_square_test(my_matrix: Vec<Vec<i64>>) -> bool {
    let n = my_matrix.len();
    let s: i64 = my_matrix[0].iter().sum();
    if my_matrix.iter().any(|row| row.iter().sum::<i64>() != s) { return false; }
    if (0..my_matrix[0].len()).any(|i| my_matrix.iter().map(|row| row[i]).sum::<i64>() != s) { return false; }
    if (0..n).map(|i| my_matrix[i][i]).sum::<i64>() != s { return false; }
    if (0..n).map(|i| my_matrix[i][n - i - 1]).sum::<i64>() != s { return false; }
    true
}
```
