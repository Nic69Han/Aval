```rust
fn lps(str1: String) -> i64 {
    let a: Vec<char> = str1.chars().collect();
    let b: Vec<char> = a.iter().rev().cloned().collect();
    let mut prev = vec![0i64; b.len() + 1];
    for x in &a {
        let mut cur = vec![0i64];
        for j in 1..=b.len() {
            let v = if b[j - 1] == *x { prev[j - 1] + 1 } else { prev[j].max(cur[j - 1]) };
            cur.push(v);
        }
        prev = cur;
    }
    prev[b.len()]
}
```
