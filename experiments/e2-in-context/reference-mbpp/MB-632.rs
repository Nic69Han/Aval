```rust
fn move_zero(num_list: Vec<i64>) -> Vec<i64> {
    let mut out: Vec<i64> = num_list.iter().cloned().filter(|&x| x != 0).collect();
    out.extend(num_list.iter().cloned().filter(|&x| x == 0));
    out
}
```
