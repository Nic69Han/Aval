```rust
fn change_base(x: i64, base: i64) -> String {
    if x < base {
        x.to_string()
    } else {
        change_base(x / base, base) + &(x % base).to_string()
    }
}
```
