```rust
fn mean_absolute_deviation(numbers: Vec<f64>) -> f64 {
    let count = numbers.len() as f64;
    let center: f64 = numbers.iter().sum::<f64>() / count;
    numbers.iter().map(|x| (x - center).abs()).sum::<f64>() / count
}
```
