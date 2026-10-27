```rust
fn remove_odd(str1: String) -> String {
    str1.chars().enumerate().filter(|(i, _)| i % 2 == 1).map(|(_, c)| c).collect()
}
```
