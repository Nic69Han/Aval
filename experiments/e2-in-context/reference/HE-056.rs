```rust
fn correct_bracketing(brackets: String) -> bool {
    let mut depth: i64 = 0;
    for c in brackets.chars() {
        if c == '<' {
            depth += 1;
        } else {
            depth -= 1;
        }
        if depth < 0 {
            return false;
        }
    }
    depth == 0
}
```
