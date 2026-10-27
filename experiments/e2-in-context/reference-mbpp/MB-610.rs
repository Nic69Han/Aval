```rust
fn remove_kth_element(list1: Vec<i64>, k: i64) -> Vec<i64> {
    let a = ((k - 1).max(0) as usize).min(list1.len());
    let b = (k.max(0) as usize).min(list1.len());
    let mut out = list1[..a].to_vec();
    out.extend_from_slice(&list1[b..]);
    out
}
```
