```rust
fn sort_sublists(input_list: Vec<Vec<String>>) -> Vec<Vec<String>> {
    input_list.into_iter().map(|mut x| { x.sort(); x }).collect()
}
```
