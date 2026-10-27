```rust
fn list_split(s: Vec<String>, step: i64) -> Vec<Vec<String>> {
    (0..step.max(0) as usize).map(|i| s.iter().skip(i).step_by(step as usize).cloned().collect()).collect()
}
```
