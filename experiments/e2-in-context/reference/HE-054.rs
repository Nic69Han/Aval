```rust
use std::collections::BTreeSet;

fn same_chars(s0: String, s1: String) -> bool {
    let a: BTreeSet<char> = s0.chars().collect();
    let b: BTreeSet<char> = s1.chars().collect();
    a == b
}
```
