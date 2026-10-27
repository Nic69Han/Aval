```rust
fn count_substrings(s: String) -> i64 {
    let ds: Vec<i64> = s.chars().map(|c| c.to_digit(10).unwrap() as i64).collect();
    let mut prefix: Vec<i64> = vec![0];
    let mut acc = 0;
    for (i, d) in ds.iter().enumerate() {
        acc += d - 1;
        let _ = i;
        prefix.push(acc);
    }
    let mut count = 0;
    for i in 0..prefix.len() {
        for j in (i + 1)..prefix.len() {
            if prefix[i] == prefix[j] { count += 1; }
        }
    }
    count
}
```
