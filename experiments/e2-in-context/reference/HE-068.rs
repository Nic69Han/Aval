```rust
fn pluck(arr: Vec<i64>) -> Vec<i64> {
    let mut best: Option<(i64, usize)> = None;
    for (i, &x) in arr.iter().enumerate() {
        if x % 2 == 0 {
            match best {
                Some((b, _)) if b <= x => {}
                _ => best = Some((x, i)),
            }
        }
    }
    match best {
        Some((v, i)) => vec![v, i as i64],
        None => vec![],
    }
}
```
