```rust
fn reverse_string_list(stringlist: Vec<String>) -> Vec<String> {
    stringlist.into_iter().map(|s| s.chars().rev().collect()).collect()
}
```
