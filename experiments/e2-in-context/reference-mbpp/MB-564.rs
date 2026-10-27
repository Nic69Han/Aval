```rust
fn count_pairs(arr: Vec<i64>, n: i64) -> i64 {
    let k = (n.max(0) as usize).min(arr.len());
    let mut c = 0;
    for i in 0..k {
        for j in i + 1..k {
            if arr[i] != arr[j] {
                c += 1;
            }
        }
    }
    c
}
```
