```rust
fn mul_even_odd(list1: Vec<i64>) -> i64 {
    let e = list1.iter().find(|x| *x % 2 == 0).unwrap();
    let o = list1.iter().find(|x| *x % 2 != 0).unwrap();
    e * o
}
```
