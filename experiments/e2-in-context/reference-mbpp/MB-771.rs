```rust
fn check_expression(exp: String) -> bool {
    let mut st: Vec<char> = Vec::new();
    for c in exp.chars() {
        match c {
            '(' | '{' | '[' => st.push(c),
            ')' | '}' | ']' => {
                let want = match c {
                    ')' => '(',
                    '}' => '{',
                    _ => '[',
                };
                if st.last() == Some(&want) {
                    st.pop();
                } else {
                    return false;
                }
            }
            _ => {}
        }
    }
    !exp.is_empty() && st.is_empty()
}
```
