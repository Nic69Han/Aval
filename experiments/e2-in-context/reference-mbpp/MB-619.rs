```rust
fn move_num(test_str: String) -> String {
    let letters: String = test_str.chars().filter(|c| !c.is_ascii_digit()).collect();
    let digits: String = test_str.chars().filter(|c| c.is_ascii_digit()).collect();
    letters + &digits
}
```
