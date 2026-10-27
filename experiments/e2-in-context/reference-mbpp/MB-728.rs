```rust
fn sum_list(lst1: Vec<i64>, lst2: Vec<i64>) -> Vec<i64> {
    lst1.iter().zip(lst2.iter()).map(|(a, b)| a + b).collect()
}
```
