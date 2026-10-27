```rust
fn total_match(lst1: Vec<String>, lst2: Vec<String>) -> Vec<String> {
    let total1: usize = lst1.iter().map(|s| s.chars().count()).sum();
    let total2: usize = lst2.iter().map(|s| s.chars().count()).sum();
    if total1 <= total2 {
        lst1
    } else {
        lst2
    }
}
```
