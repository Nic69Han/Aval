```rust
fn count_samepair(list1: Vec<i64>, list2: Vec<i64>, list3: Vec<i64>) -> i64 {
    list1.iter().zip(list2.iter()).zip(list3.iter()).filter(|((a, b), c)| a == b && b == c).count() as i64
}
```
