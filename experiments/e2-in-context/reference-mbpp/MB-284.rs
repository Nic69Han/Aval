```rust
fn check_element(list1: Vec<String>, element: String) -> bool {
    list1.iter().all(|v| *v == element)
}
```
