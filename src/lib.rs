//! Basic library for doing 3D diffs to check if two triangular meshes are the same,
//! or how different they are.
//! # Example
//! ```
//! // Let's diff these two meshes.
//! let mesh0 = camino::Utf8Path::new("testdata/cow-nonormals.obj");
//! let mesh1 = camino::Utf8Path::new("testdata/teapot.obj");
//!
//! // Run the diff:
//! let difference = thirty_thirty::diff(mesh0, mesh1).unwrap();
//!
//! // Check the results.
//! assert!(difference.max_abs_diff > 4.0);
//! assert!(difference.mean_abs_error > 2.0);
//! ```
#![deny(missing_docs)]
use camino::Utf8Path;
use rs_read_trimesh::load_trimesh;

mod math;
#[cfg(test)]
mod tests;

/// Errors that can occur
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// The file you provided could not be opened.
    /// The String field says why.
    CouldNotOpenFile(String),
    /// The meshes were empty, or had no distances available.
    /// Probably indicates an empty file or invalid data.
    EmptyDistances,
}

/// Configuration for how the difference should be calculated.
#[derive(Default, Debug, Clone, Copy)]
#[non_exhaustive]
pub struct Config {
    /// If true, the second mesh will be oriented to the same rotation as the first.
    /// False by default.
    pub reorient: bool,
}

/// Get a geometric difference between the meshes in these two files.
/// # Example
/// ```
/// // Let's diff these two meshes.
/// let mesh0 = camino::Utf8Path::new("testdata/cow-nonormals.obj");
/// let mesh1 = camino::Utf8Path::new("testdata/teapot.obj");
///
/// // Run the diff:
/// let difference = thirty_thirty::diff(mesh0, mesh1).unwrap();
///
/// // Check the results.
/// assert!(difference.max_abs_diff > 4.0);
/// assert!(difference.mean_abs_error > 2.0);
/// ```
pub fn diff(file1: &Utf8Path, file2: &Utf8Path) -> Result<Metric, Error> {
    diff_with_config(file1, file2, Default::default())
}

/// Get a geometric difference between the meshes in these two files.
/// # Example
/// ```
/// use thirty_thirty::Config;
/// use thirty_thirty::Metric;
///
/// // Let's diff these two meshes.
/// let mesh0 = camino::Utf8Path::new("testdata/cow-nonormals.obj");
/// let mesh1 = camino::Utf8Path::new("testdata/teapot.obj");
///
/// // Set the diff config.
/// let mut config = Config::default();
/// config.reorient = false;
///
/// // Run the diff:
/// let difference: Metric = thirty_thirty::diff_with_config(mesh0, mesh1, config).unwrap();
///
/// // Check the results.
/// assert!(difference.max_abs_diff > 4.0);
/// assert!(difference.mean_abs_error > 2.0);
/// ```
pub fn diff_with_config(
    file1: &Utf8Path,
    file2: &Utf8Path,
    config: Config,
) -> Result<Metric, Error> {
    let mesh1 = load_trimesh(file1.as_str(), 3.0).map_err(Error::CouldNotOpenFile)?;
    log::debug!(
        "Successfully loaded and scaled mesh with {} vertices, {} indices.",
        mesh1.vertices().len(),
        mesh1.indices().len()
    );
    let mesh2 = load_trimesh(file2.as_str(), 3.0).map_err(Error::CouldNotOpenFile)?;
    log::debug!(
        "Successfully loaded and scaled mesh with {} vertices, {} indices.",
        mesh2.vertices().len(),
        mesh2.indices().len()
    );

    math::diff_trimeshes(mesh1, mesh2, config)
}

/// Metric showing how different the two meshes were.
#[derive(Debug, PartialEq)]
pub struct Metric {
    /// Max absolute deviation
    pub max_abs_diff: f64,
    /// Mean absolute error
    pub mean_abs_error: f64,
}

impl Metric {
    /// Zero distance, i.e. the exact same.
    pub const ZERO: Self = Self {
        max_abs_diff: 0.0,
        mean_abs_error: 0.0,
    };
}
