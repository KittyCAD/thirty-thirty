use mesh_to_sdf::{AccelerationMethod, Topology, generate_sdf};
use parry3d::{
    glamx::prelude::Pose3,
    math::{Mat3, Rot3, SymmetricEigen, Vec3},
    shape::{Shape, TriMesh},
};

use crate::{Config, Error, Metric};

fn to_array(v: &Vec3) -> [f32; 3] {
    [v.x, v.y, v.z]
}

/// Get a geometric difference between these two meshes.
pub(crate) fn diff_trimeshes(
    mesh1: TriMesh,
    mesh2: TriMesh,
    config: Config,
) -> Result<Metric, Error> {
    // Diff both directions and take the max error from each.
    // This helps in a case where, say, mesh1 is a subset of mesh2.
    // You don't want to compare only the points they have in common,
    // you want to compare all points from both meshes.
    let m12 = inner_diff_trimeshes(mesh1.clone(), mesh2.clone(), config)?;
    let m21 = inner_diff_trimeshes(mesh2, mesh1, config)?;
    Ok(Metric {
        max_abs_diff: m12.max_abs_diff.max(m21.max_abs_diff),
        mean_abs_error: m12.mean_abs_error.max(m21.mean_abs_error),
    })
}

/// Get a geometric difference between these two meshes.
pub(crate) fn inner_diff_trimeshes(
    mesh1: TriMesh,
    mesh2: TriMesh,
    config: Config,
) -> Result<Metric, Error> {
    // Reorient them to face the same way, if the user asks for it.
    let (mesh1, mesh2) = if config.reorient {
        reorient(mesh1, mesh2)
    } else {
        (mesh1, mesh2)
    };

    let mesh1_vertices: Vec<[f32; 3]> = mesh1.vertices().iter().map(to_array).collect();
    let mesh2_vertices: Vec<[f32; 3]> = mesh2.vertices().iter().map(to_array).collect();

    let mesh1_triangle_indices: Vec<u32> = mesh1.indices().iter().flatten().copied().collect();

    // For every point in mesh2, find its distance from the surface of mesh1.
    let distances: Vec<f32> = generate_sdf(
        // Generate a SDF from these vertices/triangles:
        &mesh1_vertices,
        Topology::TriangleList(Some(&mesh1_triangle_indices)),
        // Then query these points against the SDF above.
        &mesh2_vertices,
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
    let (max_diff, absolute_sum) = distances.iter().map(|dist| dist.abs() as f64).fold(
        (0.0, 0.0),
        |(max_diff, absolute_sum), distance| {
            (f64::max(max_diff, distance), absolute_sum + distance)
        },
    );
    let mean_diff = absolute_sum / distances.len() as f64;

    Ok(Metric {
        max_abs_diff: max_diff,
        mean_abs_error: mean_diff,
    })
}

fn reorient(mesh1: TriMesh, mut mesh2: TriMesh) -> (TriMesh, TriMesh) {
    let t1 = compute_canonical_frame(&mesh1);
    let t2 = compute_canonical_frame(&mesh2);

    // Relative transform that maps Mesh 2 space -> Canonical frame -> Mesh 1 space
    let t_rel: Pose3 = t1 * t2.inverse();

    // Apply transformation directly to Mesh 2's vertices
    mesh2.transform_vertices(&t_rel);
    (mesh1, mesh2)
}

fn compute_canonical_frame(mesh: &TriMesh) -> Pose3 {
    // 1. Calculate mass properties
    let mass_props = mesh.mass_properties(1.0);
    let com: Vec3 = mass_props.local_com;
    let inertia_matrix = mass_props.reconstruct_inertia_matrix();

    // 2. Perform Eigendecomposition on the inertia matrix
    let eigen = SymmetricEigen::new(inertia_matrix);

    let mut axes = [
        eigen.eigenvectors.col(0),
        eigen.eigenvectors.col(1),
        eigen.eigenvectors.col(2),
    ];

    // 3. Orient ALL THREE axes using geometric skewness
    let mut skews = [0.0; 3];
    for i in 0..3 {
        let (oriented_axis, skew_mag) = fix_axis_orientation(mesh, com, axes[i]);
        axes[i] = oriented_axis;
        skews[i] = skew_mag;
    }

    let mut x_axis = axes[0];
    let mut y_axis = axes[1];
    let mut z_axis = axes[2];

    // 4. Ensure a right-handed coordinate frame (det(R) == +1)
    // If det < 0, flip the axis with the smallest skewness magnitude (the most symmetric axis)
    let rot_mat = Mat3::from_cols(x_axis, y_axis, z_axis);
    if rot_mat.determinant() < 0.0 {
        let mut min_idx = 0;
        if skews[1] < skews[min_idx] {
            min_idx = 1;
        }
        if skews[2] < skews[min_idx] {
            min_idx = 2;
        }

        match min_idx {
            0 => x_axis = -x_axis,
            1 => y_axis = -y_axis,
            2 => z_axis = -z_axis,
            _ => unreachable!(),
        }
    }

    let rot_mat = Mat3::from_cols(x_axis, y_axis, z_axis);
    let rotation = Rot3::from_mat3(&rot_mat);

    Pose3::from_parts(com, rotation)
}

fn fix_axis_orientation(mesh: &TriMesh, com: Vec3, mut axis: Vec3) -> (Vec3, f32) {
    let mut skewness = 0.0;

    for pt in mesh.vertices() {
        let vec_from_com = *pt - com;
        let proj = vec_from_com.dot(axis);
        // Cubing preserves the sign and highlights asymmetry
        skewness += proj.powi(3);
    }

    // Epsilon threshold avoids random flips from machine noise on symmetric axes (~0.0)
    const EPSILON: f32 = 1e-4;

    if skewness < -EPSILON {
        axis = -axis;
        skewness = -skewness;
    }

    (axis, skewness.abs())
}
