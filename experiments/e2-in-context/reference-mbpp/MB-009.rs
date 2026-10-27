```rust
fn find_rotations(s: String) -> i64 {
    let cs: Vec<char> = s.chars().collect();
    let n = cs.len();
    let mut twice = cs.clone();
    twice.extend(cs.iter());
    for i in 1..=n {
        if twice[i..i + n] == cs[..] { return i as i64; }
    }
    n as i64
}
```
