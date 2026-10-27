```rust
fn triples_sum_to_zero(numbers: Vec<i64>) -> bool {
    let n = numbers.len();
    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                if numbers[i] + numbers[j] + numbers[k] == 0 {
                    return true;
                }
            }
        }
    }
    false
}
```
