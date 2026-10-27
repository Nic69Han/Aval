```rust
fn find_substring(str1: Vec<String>, sub_str: String) -> bool {
    str1.iter().any(|w| w.contains(sub_str.as_str()))
}
```
