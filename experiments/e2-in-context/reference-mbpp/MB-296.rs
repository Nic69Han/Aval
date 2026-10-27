```rust
fn get_inv_count(arr: Vec<i64>) -> i64 {
    let mut count = 0;
    for i in 0..arr.len() {
        for j in (i + 1)..arr.len() {
            if arr[i] > arr[j] { count += 1; }
        }
    }
    count
}
```
