use camino::Utf8Path;

use crate::{Config, Metric};

#[derive(Eq, PartialEq, Debug)]
enum TestCase {
    Cow,
    Pumpkin,
    Teapot,
    Teddy,
}

impl TestCase {
    fn load(&self) -> &'static Utf8Path {
        let s = match self {
            TestCase::Cow => "testdata/cow-nonormals.obj",
            TestCase::Pumpkin => "testdata/pumpkin_tall_10k.obj",
            TestCase::Teapot => "testdata/teapot.obj",
            TestCase::Teddy => "testdata/teddy.obj",
        };
        Utf8Path::new(s)
    }

    fn all() -> Vec<Self> {
        vec![Self::Cow, Self::Pumpkin, Self::Teapot, Self::Teddy]
    }
}

/// Two identical meshes that have been rotated should have a difference metric of 0,
/// IF the opt-in option for reorienting models is true.
#[test]
fn rotation_indifference() {
    let tri1 = Utf8Path::new("testdata/triangle1.obj");
    let tri2 = Utf8Path::new("testdata/triangle2.obj");
    let metric = crate::diff_with_config(
        tri1,
        tri2,
        Config {
            reorient: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        metric.mean_abs_error < 0.000001,
        "Identical meshes that are rotated should have approximately zero mean difference when using `reorient: true`, but they actually had {}",
        metric.mean_abs_error,
    );
}

/// Two identical meshes that have been rotated should have a positive difference metric.
#[test]
fn rotation_difference() {
    let tri1 = Utf8Path::new("testdata/triangle1.obj");
    let tri2 = Utf8Path::new("testdata/triangle2.obj");
    let metric = crate::diff_with_config(tri1, tri2, Config::default()).unwrap();
    assert!(
        metric.mean_abs_error > 1.0,
        "Identical meshes that are rotated should be considered different",
    );
}

/// Basic properties:
/// - any identical files should have difference metric of 0.
/// - any non-identical files should have a difference metric > 0.
#[test]
fn basic() {
    let all = TestCase::all();
    for t0 in &all {
        for t1 in &all {
            let metric = crate::diff(t0.load(), t1.load()).expect("Diffing these should succeed");
            if t0 == t1 {
                assert_eq!(
                    metric,
                    Metric::ZERO,
                    "These two test cases are the same, so their difference should be zero."
                );
            } else {
                assert_ne!(
                    metric,
                    Metric::ZERO,
                    "These two test cases are different, so their difference should be nonzero."
                );
                assert!(
                    metric.max_abs_diff > 0.0,
                    "These two test cases are different, so their max absolute difference should be positive."
                );
                assert!(
                    metric.mean_abs_error > 0.0,
                    "These two test cases are different, so their mean absolute error should be positive."
                );
            }
        }
    }
}
