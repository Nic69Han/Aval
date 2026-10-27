```rust
fn bell_number(n: i64) -> i64 {
    let mut r: Vec<i64> = vec![1];
    for _ in 0..n {
        let mut next = Vec::new();
        for k in 0..=r.len() {
            let a = if k < r.len() { k as i64 * r[k] } else { 0 };
            let b = if k >= 1 { r[k - 1] } else { 0 };
            next.push(a + b);
        }
        r = next;
    }
    r.iter().sum()
}
```
