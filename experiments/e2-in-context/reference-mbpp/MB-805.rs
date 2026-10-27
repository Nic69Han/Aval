```rust
fn max_sum_list(lists: Vec<Vec<i64>>) -> Vec<i64> {
    let m = lists.iter().map(|l| l.iter().sum::<i64>()).max().unwrap();
    lists.into_iter().find(|l| l.iter().sum::<i64>() == m).unwrap()
}
```
