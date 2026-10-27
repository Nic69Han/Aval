```rust
fn find_kth(arr1: Vec<i64>, arr2: Vec<i64>, k: i64) -> i64 {
    let mut v = arr1;
    v.extend(arr2);
    v.sort();
    v[(k - 1) as usize]
}
```
