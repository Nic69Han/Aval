```rust
fn text_match_wordz_middle(text: String) -> bool {
    let w = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let cs: Vec<char> = text.chars().collect();
    (1..cs.len()).any(|i| i + 1 < cs.len() && cs[i] == 'z' && w(cs[i - 1]) && w(cs[i + 1]))
}
```
