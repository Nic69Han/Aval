```rust
fn remove_duplicates(numbers: Vec<i64>) -> Vec<i64> {
    numbers
        .iter()
        .filter(|x| numbers.iter().filter(|y| y == x).count() == 1)
        .cloned()
        .collect()
}
```
