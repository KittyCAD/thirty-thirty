use camino::Utf8Path;
use mesh_to_sdf::{AccelerationMethod, Topology, generate_sdf};
use parry3d::shape::TriMesh;
use rs_read_trimesh::load_trimesh;

/// Errors that can occur
pub enum Error {
    CouldNotOpenFile(String),
    EmptyDistances,
}

/// Get a geometric difference between the meshes in these two files.
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

    let distances: Vec<f32> = generate_sdf(
        &mesh1v,
        Topology::TriangleList(Some(&mesh1_indices)),
        &mesh2v, // Query points must match the vertex primitive shape types
        AccelerationMethod::RtreeBvh, // Optimal blend for processing dense geometry
    );

    if distances.is_empty() {
        return Err(Error::EmptyDistances);
    }

    let (max_diff, absolute_sum) = distances
        .iter()
        .map(|dist| dist.abs())
        .fold((0.0, 0.0), |(diff, sum), e| (f32::max(diff, e), sum + e));
    let mean_diff = absolute_sum / distances.len() as f32;

    Ok(Metric {
        max_abs_diff: max_diff,
        mean_abs_error: mean_diff,
    })
}

/// Metric showing how different the two meshes were.
pub struct Metric {
    /// Max absolute deviation
    pub max_abs_diff: f32,
    /// Mean absolute error
    pub mean_abs_error: f32,
}
