```rust
fn strange_sort_list(lst: Vec<i64>) -> Vec<i64> {
    let mut s = lst.clone();
    s.sort();
    let n = s.len();
    (0..n)
        .map(|i| if i % 2 == 0 { s[i / 2] } else { s[n - 1 - i / 2] })
        .collect()
}
```
