```rust
fn round_and_sum(list1: Vec<f64>) -> i64 {
    fn py_round(x: f64) -> i64 {
        let f = x.floor();
        let diff = x - f;
        if diff < 0.5 { f as i64 } else if diff > 0.5 { f as i64 + 1 } else if (f as i64) % 2 == 0 { f as i64 } else { f as i64 + 1 }
    }
    list1.iter().map(|x| py_round(*x)).sum::<i64>() * list1.len() as i64
}
```
