```rust
fn fib4(n: i64) -> i64 {
    if n < 4 {
        return [0, 0, 2, 0][n as usize];
    }
    let mut state: [i64; 4] = [0, 0, 2, 0];
    for _ in 0..(n - 3) {
        let next = state[0] + state[1] + state[2] + state[3];
        state = [state[1], state[2], state[3], next];
    }
    state[3]
}
```
