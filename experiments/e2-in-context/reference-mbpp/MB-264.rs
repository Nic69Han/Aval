```rust
fn dog_age(h_age: i64) -> f64 {
    if h_age <= 2 { h_age as f64 * 10.5 } else { (21 + (h_age - 2) * 4) as f64 }
}
```
