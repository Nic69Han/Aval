```rust
fn count_reverse_pairs(test_list: Vec<String>) -> i64 {
    let n = test_list.len();
    let mut c = 0;
    for i in 0..n {
        let r: String = test_list[i].chars().rev().collect();
        for j in i + 1..n {
            if test_list[j] == r {
                c += 1;
            }
        }
    }
    c
}
```
