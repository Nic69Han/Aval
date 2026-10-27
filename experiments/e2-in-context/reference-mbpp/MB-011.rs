```rust
fn remove_occ(s: String, ch: String) -> String {
    let cs: Vec<char> = s.chars().collect();
    let pos: Vec<usize> = (0..cs.len()).filter(|&i| ch.chars().count() == 1 && cs[i] == ch.chars().next().unwrap()).collect();
    if pos.is_empty() { return s; }
    let first = pos[0];
    let last = pos[pos.len() - 1];
    (0..cs.len()).filter(|&i| i != first && i != last).map(|i| cs[i]).collect()
}
```
