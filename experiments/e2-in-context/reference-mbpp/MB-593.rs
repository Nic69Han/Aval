```rust
fn removezero_ip(ip: String) -> String {
    ip.split('.')
        .enumerate()
        .map(|(i, p)| if i == 0 { p.to_string() } else { p.trim_start_matches('0').to_string() })
        .collect::<Vec<_>>()
        .join(".")
}
```
