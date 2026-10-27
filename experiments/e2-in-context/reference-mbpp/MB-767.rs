```rust
fn get_pairs_count(arr: Vec<i64>, sum_: i64) -> f64 {
    let mut c = 0;
    for i in 0..arr.len() {
        for j in i + 1..arr.len() {
            if arr[i] + arr[j] == sum_ {
                c += 1;
            }
        }
    }
    c as f64
}
```
