```rust
fn shell_sort(my_list: Vec<i64>) -> Vec<i64> {
    let mut v = my_list;
    let mut gap = v.len() / 2;
    while gap > 0 {
        for i in gap..v.len() {
            let t = v[i];
            let mut j = i;
            while j >= gap && v[j - gap] > t {
                v[j] = v[j - gap];
                j -= gap;
            }
            v[j] = t;
        }
        gap /= 2;
    }
    v
}
```
