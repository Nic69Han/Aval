```rust
fn eulerian_num(n: i64, m: i64) -> i64 {
    if n <= 0 || m < 0 || m >= n { return 0; }
    let mut row: Vec<i64> = vec![1];
    for k in 2..=n {
        let mut next = Vec::new();
        for j in 0..k {
            let up = row[(j - 1).max(0) as usize];
            let same = if (j as usize) < row.len() { row[j as usize] } else { 0 };
            next.push(if j == 0 { 1 } else { (k - j) * up + (j + 1) * same });
        }
        row = next;
    }
    row[m as usize]
}
```
