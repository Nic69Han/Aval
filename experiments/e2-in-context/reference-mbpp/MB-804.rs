```rust
fn is_product_even(arr: Vec<i64>) -> bool {
    arr.iter().any(|x| x % 2 == 0)
}
```
