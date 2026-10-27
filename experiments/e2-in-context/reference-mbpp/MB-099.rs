```rust
fn decimal_to_binary(n: i64) -> String {
    format!("{}{:b}", if n < 0 { "-" } else { "" }, n.abs())
}
```
