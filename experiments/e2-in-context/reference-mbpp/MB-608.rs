```rust
fn bell_number(n: i64) -> i64 {
    let mut row: Vec<i64> = vec![1];
    for _ in 1..n {
        let mut next = vec![*row.last().unwrap()];
        for &x in &row {
            let v = next.last().unwrap() + x;
            next.push(v);
        }
        row = next;
    }
    *row.last().unwrap()
}
```
