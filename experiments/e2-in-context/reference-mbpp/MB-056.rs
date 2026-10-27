```rust
fn check(n: i64) -> bool {
    let rev: i64 = n.to_string().chars().rev().collect::<String>().parse().unwrap();
    n == 2 * rev - 1
}
```
