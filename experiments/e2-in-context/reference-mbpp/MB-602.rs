```rust
fn first_repeated_char(str1: String) -> String {
    let cs: Vec<char> = str1.chars().collect();
    for i in 0..cs.len() {
        if cs[..i].contains(&cs[i]) {
            return cs[i].to_string();
        }
    }
    String::new()
}
```
