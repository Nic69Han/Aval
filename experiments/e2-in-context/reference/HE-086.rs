```rust
fn anti_shuffle(s: String) -> String {
    s.split(' ')
        .map(|w| {
            let mut cs: Vec<char> = w.chars().collect();
            cs.sort();
            cs.into_iter().collect::<String>()
        })
        .collect::<Vec<String>>()
        .join(" ")
}
```
