```rust
fn unique_element(arr: Vec<i64>) -> bool {
    !arr.is_empty() && arr.iter().all(|&x| x == arr[0])
}
```
