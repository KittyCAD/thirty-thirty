use camino::Utf8Path;

use crate::{Config, Metric};

const TEST_EPSILON: f64 = 0.000001;

#[derive(Eq, PartialEq, Debug)]
enum ObjTestCase {
    Cow,
    Pumpkin,
    Teapot,
    Teddy,
}

impl ObjTestCase {
    fn load(&self) -> &'static Utf8Path {
        let s = match self {
            ObjTestCase::Cow => "testdata/cow-nonormals.obj",
            ObjTestCase::Pumpkin => "testdata/pumpkin_tall_10k.obj",
            ObjTestCase::Teapot => "testdata/teapot.obj",
            ObjTestCase::Teddy => "testdata/teddy.obj",
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
        metric.mean_abs_error < TEST_EPSILON,
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

#[test]
fn gltf_check_identical() {
    let tri1 = Utf8Path::new("testdata/triangle1ish.glb");
    let actual_metric = crate::diff_with_config(tri1, tri1, Config::default()).unwrap();
    assert_eq!(
        actual_metric,
        Metric::ZERO,
        "identical files should have distance 0"
    );
}

#[test]
fn check_subsets() {
    // This is a subset of the following GLB.
    let d_sans_box = Utf8Path::new("testdata/dshape.glb");
    let d_with_box = Utf8Path::new("testdata/d_with_box.glb");
    let delta1 = crate::diff_with_config(d_sans_box, d_with_box, Config::default()).unwrap();
    let delta2 = crate::diff_with_config(d_with_box, d_sans_box, Config::default()).unwrap();
    dbg!(&delta1);
    dbg!(&delta2);
    let epsilon = 0.01;
    assert!(
        delta1.max_abs_diff > epsilon,
        "the presence or absence of the box should matter, but their delta was {delta1:?}"
    );
    assert!(
        delta2.max_abs_diff > epsilon,
        "the presence or absence of the box should matter, but their delta was {delta2:?}"
    );
}

#[test]
fn gltf_check_different() {
    let tri_glb = Utf8Path::new("testdata/triangle1ish.glb");
    let d_glb = Utf8Path::new("testdata/dshape.glb");

    // Different models, so their difference should be positive.
    let actual = crate::diff(tri_glb, d_glb).unwrap().mean_abs_error;
    assert!(actual > 0.002);
}

/// Basic properties:
/// - any identical files should have difference metric of 0.
/// - any non-identical files should have a difference metric > 0.
#[test]
fn basic() {
    let all = ObjTestCase::all();
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
                    metric.max_abs_diff > 1.0,
                    "These two test cases are different, so their max absolute difference should be positive."
                );
                assert!(
                    metric.mean_abs_error > 1.0,
                    "These two test cases are different, so their mean absolute error should be positive."
                );
            }
        }
    }
}
