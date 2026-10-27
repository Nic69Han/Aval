```rust
fn pairs_sum_to_zero(numbers: Vec<i64>) -> bool {
    for i in 0..numbers.len() {
        for j in (i + 1)..numbers.len() {
            if numbers[i] + numbers[j] == 0 {
                return true;
            }
        }
    }
    false
}
```
