```rust
fn add_string(list_: Vec<i64>, string: String) -> Vec<String> {
    list_.iter().map(|v| string.replace("{0}", &v.to_string())).collect()
}
```
