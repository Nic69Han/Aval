```rust
fn multiply_num(numbers: Vec<f64>) -> f64 {
    numbers.iter().product::<f64>() / numbers.len() as f64
}
```
