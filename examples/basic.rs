use thirty_thirty::Metric;

fn main() {
    // Let's diff these two meshes.
    let mesh0 = camino::Utf8Path::new("testdata/cow-nonormals.obj");
    let mesh1 = camino::Utf8Path::new("testdata/teapot.obj");

    // Run the diff:
    let difference: Metric = thirty_thirty::diff(mesh0, mesh1).unwrap();

    // Check the results.
    assert_eq!(difference.max_abs_diff, 4.3111653);
    assert_eq!(difference.mean_abs_error, 2.1293483);

    // If we diff a mesh against itself, its difference will be zero.
    assert_eq!(thirty_thirty::diff(mesh0, mesh0).unwrap(), Metric::ZERO);
}
