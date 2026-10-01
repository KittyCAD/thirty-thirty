use camino::Utf8Path;

use crate::Metric;

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
