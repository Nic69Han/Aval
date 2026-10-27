```rust
fn get_ludic(n: i64) -> Vec<i64> {
    let mut out = vec![1];
    let mut l: Vec<i64> = (2..=n).collect();
    while !l.is_empty() {
        let h = l[0] as usize;
        out.push(l[0]);
        l = l.iter().enumerate().filter(|(i, _)| i % h != 0).map(|(_, &x)| x).collect();
    }
    out
}
```
