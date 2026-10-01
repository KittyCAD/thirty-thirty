# thirty-thirty

Like <https://github.com/KittyCAD/twenty-twenty>, but in 3D


## Example

```rust
fn main() {
    // Let's diff these two meshes.
    let mesh0 = camino::Utf8Path::new("testdata/cow-nonormals.obj");
    let mesh1 = camino::Utf8Path::new("testdata/teapot.obj");

    // Run the diff:
    let difference = thirty_thirty::diff(mesh0, mesh1).unwrap();

    // Check the results.
    assert_eq!(difference.max_abs_diff, 4.3111653);
    assert_eq!(difference.mean_abs_error, 2.1293483);
}
```
