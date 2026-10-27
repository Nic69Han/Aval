```rust
fn find_length(string: String) -> i64 {
    let mut cur = 0i64;
    let mut best = 0i64;
    for c in string.chars() {
        cur += if c == '0' { 1 } else { -1 };
        if cur < 0 { cur = 0; }
        best = best.max(cur);
    }
    best
}
```
