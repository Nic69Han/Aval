```rust
fn max_sum(arr: Vec<i64>) -> i64 {
    fn msibs(arr: &[i64]) -> Vec<i64> {
        let mut m = arr.to_vec();
        for i in 0..arr.len() {
            let mut best = 0;
            for j in 0..i {
                if arr[j] < arr[i] { best = best.max(m[j]); }
            }
            m[i] = arr[i] + best;
        }
        m
    }
    let inc = msibs(&arr);
    let mut rev = arr.clone();
    rev.reverse();
    let mut dec = msibs(&rev);
    dec.reverse();
    (0..arr.len()).map(|i| inc[i] + dec[i] - arr[i]).max().unwrap()
}
```
