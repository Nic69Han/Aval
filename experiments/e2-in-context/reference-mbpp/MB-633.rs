```rust
fn pair_xor_sum(arr: Vec<i64>, n: i64) -> i64 {
    let k = (n.max(0) as usize).min(arr.len());
    let mut s = 0;
    for i in 0..k {
        for j in i + 1..k {
            s += arr[i] ^ arr[j];
        }
    }
    s
}
```
