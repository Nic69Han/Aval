```rust
use std::collections::HashSet;

fn count_distinct_characters(string: String) -> i64 {
    let set: HashSet<char> = string.to_lowercase().chars().collect();
    set.len() as i64
}
```
