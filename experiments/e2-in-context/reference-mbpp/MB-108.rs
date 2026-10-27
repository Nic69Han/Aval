```rust
fn merge_sorted_list(num1: Vec<f64>, num2: Vec<f64>, num3: Vec<f64>) -> Vec<f64> {
    let mut all = num1.clone();
    all.extend(num2);
    all.extend(num3);
    all.sort_by(|a, b| a.partial_cmp(b).unwrap());
    all
}
```
