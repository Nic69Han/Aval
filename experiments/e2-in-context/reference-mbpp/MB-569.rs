```rust
fn sort_sublists(list1: Vec<Vec<String>>) -> Vec<Vec<String>> {
    list1
        .into_iter()
        .map(|mut s| {
            s.sort();
            s
        })
        .collect()
}
```
