```rust
fn is_samepatterns(colors: Vec<String>, patterns: Vec<String>) -> bool {
    if colors.len() != patterns.len() { return false; }
    for i in 0..colors.len() {
        for j in 0..colors.len() {
            if patterns[i] == patterns[j] && colors[i] != colors[j] { return false; }
        }
    }
    true
}
```
