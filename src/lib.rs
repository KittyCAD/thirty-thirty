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
//! assert_eq!(difference.max_abs_diff, 4.3111653);
//! assert_eq!(difference.mean_abs_error, 2.1293483);
//! ```
use camino::Utf8Path;
use mesh_to_sdf::{AccelerationMethod, Topology, generate_sdf};
use parry3d::shape::TriMesh;
use rs_read_trimesh::load_trimesh;

#[cfg(test)]
mod tests;

/// Errors that can occur
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    CouldNotOpenFile(String),
    EmptyDistances,
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
/// assert_eq!(difference.max_abs_diff, 4.3111653);
/// assert_eq!(difference.mean_abs_error, 2.1293483);
/// ```
pub fn diff(file1: &Utf8Path, file2: &Utf8Path) -> Result<Metric, Error> {
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

    diff_trimeshes(mesh1, mesh2)
}

/// Get a geometric difference between these two meshes.
fn diff_trimeshes(mesh1: TriMesh, mesh2: TriMesh) -> Result<Metric, Error> {
    let mesh1v: Vec<[f32; 3]> = mesh1.vertices().iter().map(|v| [v.x, v.y, v.z]).collect();
    let mesh2v: Vec<[f32; 3]> = mesh2.vertices().iter().map(|v| [v.x, v.y, v.z]).collect();

    let mesh1_indices: Vec<u32> = mesh1.indices().iter().flatten().copied().collect();

    // For every point in mesh2, find its distance from the surface of mesh1.
    let distances: Vec<f32> = generate_sdf(
        // Generate a SDF from these vertices/triangles:
        &mesh1v,
        Topology::TriangleList(Some(&mesh1_indices)),
        // Then query these points against the SDF above.
        &mesh2v,
        // Chosen arbitrarily based on what a hungry ghost in a jar said to do.
        AccelerationMethod::RtreeBvh,
    );

    // Should not happen, does not have a meaningful distance,
    // so tell the user and the user can handle it.
    if distances.is_empty() {
        return Err(Error::EmptyDistances);
    }

    // From those distances, compute some useful metrics the user
    // might want to know.
    let (max_diff, absolute_sum) = distances.iter().map(|dist| dist.abs()).fold(
        (0.0, 0.0),
        |(max_diff, absolute_sum), distance| {
            (f32::max(max_diff, distance), absolute_sum + distance)
        },
    );
    let mean_diff = absolute_sum / distances.len() as f32;

    Ok(Metric {
        max_abs_diff: max_diff,
        mean_abs_error: mean_diff,
    })
}

/// Metric showing how different the two meshes were.
#[derive(Debug, PartialEq)]
pub struct Metric {
    /// Max absolute deviation
    pub max_abs_diff: f32,
    /// Mean absolute error
    pub mean_abs_error: f32,
}

impl Metric {
    /// Zero distance, i.e. the exact same.
    pub const ZERO: Self = Self {
        max_abs_diff: 0.0,
        mean_abs_error: 0.0,
    };
}
