```rust
fn remove_lowercase(str1: String) -> String {
    str1.chars().filter(|c| !c.is_ascii_lowercase()).collect()
}
```
