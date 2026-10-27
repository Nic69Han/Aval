```rust
fn check_min_heap(arr: Vec<i64>) -> bool {
    let n = arr.len();
    (0..n).all(|i| (2 * i + 1 >= n || arr[i] <= arr[2 * i + 1]) && (2 * i + 2 >= n || arr[i] <= arr[2 * i + 2]))
}
```
