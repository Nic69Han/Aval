```rust
fn lateralsurface_cone(r: f64, h: f64) -> f64 {
    std::f64::consts::PI * r * (r * r + h * h).sqrt()
}
```
