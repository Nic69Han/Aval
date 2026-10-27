```rust
fn capital_words_spaces(str1: String) -> String {
    let w = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let cs: Vec<char> = str1.chars().collect();
    let mut flags = vec![false; cs.len()];
    for i in 1..cs.len() {
        flags[i] = cs[i].is_ascii_uppercase() && w(cs[i - 1]) && !flags[i - 1];
    }
    let mut out = String::new();
    for i in 0..cs.len() {
        if flags[i] {
            out.push(' ');
        }
        out.push(cs[i]);
    }
    out
}
```
