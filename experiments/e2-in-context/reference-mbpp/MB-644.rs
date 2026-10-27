```rust
fn reverse_array_upto_k(input: Vec<i64>, k: i64) -> Vec<i64> {
    let m = (k.max(0) as usize).min(input.len());
    let mut out: Vec<i64> = input[..m].iter().rev().cloned().collect();
    out.extend_from_slice(&input[m..]);
    out
}
```
