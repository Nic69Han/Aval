```rust
fn get_max_sum(n: i64) -> i64 {
    fn f(n: i64) -> i64 {
        if n < 1 { return 0; }
        (f(n / 2) + f(n / 3) + f(n / 4) + f(n / 5)).max(n)
    }
    f(n)
}
```
