```rust
fn fruit_distribution(s: String, n: i64) -> i64 {
    let words: Vec<&str> = s.split(' ').collect();
    let apples: i64 = words[0].parse().unwrap();
    let oranges: i64 = words[3].parse().unwrap();
    n - apples - oranges
}
```
