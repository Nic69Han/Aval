```rust
fn is_palindrome(text: String) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let mut reversed = chars.clone();
    reversed.reverse();
    chars == reversed
}
```
