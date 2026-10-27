```rust
fn extract_index_list(l1: Vec<i64>, l2: Vec<i64>, l3: Vec<i64>) -> Vec<f64> {
    let m = l1.len().min(l2.len()).min(l3.len());
    (0..m).filter(|&i| l1[i] == l2[i] && l2[i] == l3[i]).map(|i| l1[i] as f64).collect()
}
```
