```rust
fn amicable_numbers_sum(limit: i64) -> i64 {
    fn div_sum(num: i64) -> i64 {
        let mut res = 1;
        let mut i = 2;
        while i * i <= num {
            if num % i == 0 {
                res += i;
                if i * i != num { res += num / i; }
            }
            i += 1;
        }
        res
    }
    (2..=limit).filter(|&i| div_sum(div_sum(i)) == i && div_sum(i) != i).sum()
}
```
