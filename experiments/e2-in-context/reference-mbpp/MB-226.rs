```rust
fn odd_values_string(str1: String) -> String {
    str1.chars().enumerate().filter(|(i, _)| i % 2 == 0).map(|(_, c)| c).collect()
}
```
