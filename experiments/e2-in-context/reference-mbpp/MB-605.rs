```rust
fn prime_num(num: i64) -> bool {
    if num < 2 {
        return false;
    }
    let mut d = 2;
    while d * d <= num {
        if num % d == 0 {
            return false;
        }
        d += 1;
    }
    true
}
```
