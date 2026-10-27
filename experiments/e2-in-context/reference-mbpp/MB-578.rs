```rust
fn interleave_lists(list1: Vec<i64>, list2: Vec<i64>, list3: Vec<i64>) -> Vec<i64> {
    let m = list1.len().min(list2.len()).min(list3.len());
    let mut out = Vec::new();
    for i in 0..m {
        out.push(list1[i]);
        out.push(list2[i]);
        out.push(list3[i]);
    }
    out
}
```
