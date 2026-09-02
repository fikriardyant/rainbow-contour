use crate::dxf::{Point3D, Triangle3D};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridPointDelta {
    pub x: f64,
    pub y: f64,
    pub z_topo: f64,
    pub z_design: f64,
    pub delta_z: f64,
}

pub fn point_in_polygon(x: f64, y: f64, poly: &[Point3D]) -> bool {
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        if ((poly[i].y > y) != (poly[j].y > y))
            && (x < (poly[j].x - poly[i].x) * (y - poly[i].y) / (poly[j].y - poly[i].y) + poly[i].x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Calculate barycentric elevation Z at (x, y) if inside triangle
pub fn interpolate_triangle_z(x: f64, y: f64, tri: &Triangle3D) -> Option<f64> {
    let (x1, y1, z1) = (tri.v0.x, tri.v0.y, tri.v0.z);
    let (x2, y2, z2) = (tri.v1.x, tri.v1.y, tri.v1.z);
    let (x3, y3, z3) = (tri.v2.x, tri.v2.y, tri.v2.z);

    let det = (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3);
    if det.abs() < 1e-9 {
        return None;
    }

    let w1 = ((y2 - y3) * (x - x3) + (x3 - x2) * (y - y3)) / det;
    let w2 = ((y3 - y1) * (x - x3) + (x1 - x3) * (y - y3)) / det;
    let w3 = 1.0 - w1 - w2;

    let eps = -1e-4;
    if w1 >= eps && w2 >= eps && w3 >= eps {
        Some(w1 * z1 + w2 * z2 + w3 * z3)
    } else {
        None
    }
}

/// Spatial index for fast triangle surface lookups
pub struct SpatialMeshIndex<'a> {
    cell_size: f64,
    cells: HashMap<(i64, i64), Vec<&'a Triangle3D>>,
}

impl<'a> SpatialMeshIndex<'a> {
    pub fn new(mesh: &'a [Triangle3D], cell_size: f64) -> Self {
        let mut cells: HashMap<(i64, i64), Vec<&'a Triangle3D>> = HashMap::new();
        for tri in mesh {
            let min_x = tri.v0.x.min(tri.v1.x).min(tri.v2.x);
            let max_x = tri.v0.x.max(tri.v1.x).max(tri.v2.x);
            let min_y = tri.v0.y.min(tri.v1.y).min(tri.v2.y);
            let max_y = tri.v0.y.max(tri.v1.y).max(tri.v2.y);

            let min_gx = (min_x / cell_size).floor() as i64;
            let max_gx = (max_x / cell_size).floor() as i64;
            let min_gy = (min_y / cell_size).floor() as i64;
            let max_gy = (max_y / cell_size).floor() as i64;

            for gx in min_gx..=max_gx {
                for gy in min_gy..=max_gy {
                    cells.entry((gx, gy)).or_default().push(tri);
                }
            }
        }
        Self { cell_size, cells }
    }

    pub fn interpolate_z(&self, x: f64, y: f64) -> Option<f64> {
        let gx = (x / self.cell_size).floor() as i64;
        let gy = (y / self.cell_size).floor() as i64;

        if let Some(triangles) = self.cells.get(&(gx, gy)) {
            for tri in triangles {
                let min_x = tri.v0.x.min(tri.v1.x).min(tri.v2.x);
                let max_x = tri.v0.x.max(tri.v1.x).max(tri.v2.x);
                let min_y = tri.v0.y.min(tri.v1.y).min(tri.v2.y);
                let max_y = tri.v0.y.max(tri.v1.y).max(tri.v2.y);

                if x >= min_x && x <= max_x && y >= min_y && y <= max_y {
                    if let Some(z) = interpolate_triangle_z(x, y, tri) {
                        return Some(z);
                    }
                }
            }
        }
        None
    }
}

pub fn compute_grid_delta(
    topo: &[Triangle3D],
    design: &[Triangle3D],
    boundary: &[Point3D],
    step: f64,
) -> Vec<GridPointDelta> {
    if topo.is_empty() || design.is_empty() {
        return Vec::new();
    }

    // Determine bounding box from boundary or design
    let has_valid_boundary = !boundary.is_empty() && !(boundary.len() <= 4 && boundary[0].x == 0.0 && boundary[1].x == 100.0);

    let (min_x, max_x, min_y, max_y) = if has_valid_boundary {
        (
            boundary.iter().map(|p| p.x).fold(f64::INFINITY, f64::min),
            boundary.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max),
            boundary.iter().map(|p| p.y).fold(f64::INFINITY, f64::min),
            boundary.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max),
        )
    } else {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for tri in design {
            min_x = min_x.min(tri.v0.x).min(tri.v1.x).min(tri.v2.x);
            max_x = max_x.max(tri.v0.x).max(tri.v1.x).max(tri.v2.x);
            min_y = min_y.min(tri.v0.y).min(tri.v1.y).min(tri.v2.y);
            max_y = max_y.max(tri.v0.y).max(tri.v1.y).max(tri.v2.y);
        }
        (min_x, max_x, min_y, max_y)
    };

    if min_x > max_x || min_y > max_y {
        return Vec::new();
    }

    // Build spatial hash indices for fast O(1) point-in-mesh lookups
    let index_cell_size = (step * 10.0).max(20.0);
    let topo_index = SpatialMeshIndex::new(topo, index_cell_size);
    let design_index = SpatialMeshIndex::new(design, index_cell_size);

    let mut points = Vec::new();
    let mut curr_y = min_y;
    while curr_y <= max_y {
        let mut curr_x = min_x;
        while curr_x <= max_x {
            if !has_valid_boundary || point_in_polygon(curr_x, curr_y, boundary) {
                let z_t = topo_index.interpolate_z(curr_x, curr_y);
                let z_d = design_index.interpolate_z(curr_x, curr_y);

                if let (Some(zt), Some(zd)) = (z_t, z_d) {
                    points.push(GridPointDelta {
                        x: curr_x,
                        y: curr_y,
                        z_topo: zt,
                        z_design: zd,
                        delta_z: zd - zt,
                    });
                }
            }
            curr_x += step;
        }
        curr_y += step;
    }

    points
}
