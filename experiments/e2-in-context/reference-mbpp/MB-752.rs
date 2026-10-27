```rust
fn jacobsthal_num(n: i64) -> i64 {
    let mut js: Vec<i64> = vec![0, 1];
    for i in 2..=n as usize {
        let v = js[i - 1] + 2 * js[i - 2];
        js.push(v);
    }
    js[n as usize]
}
```
