```rust
fn reverse_vowels(str1: String) -> String {
    let vowels = "aeiouAEIOU";
    let cs: Vec<char> = str1.chars().collect();
    let mut vs: Vec<char> = cs.iter().cloned().filter(|c| vowels.contains(*c)).collect();
    vs.reverse();
    let mut k = 0;
    cs.iter()
        .map(|c| if vowels.contains(*c) { k += 1; vs[k - 1] } else { *c })
        .collect()
}
```
