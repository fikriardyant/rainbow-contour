use crate::dxf::{Point3D, Triangle3D};
use serde::{Deserialize, Serialize};

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

pub fn compute_grid_delta(
    topo: &[Triangle3D],
    design: &[Triangle3D],
    boundary: &[Point3D],
    step: f64,
) -> Vec<GridPointDelta> {
    let min_x = boundary.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let max_x = boundary.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let min_y = boundary.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let max_y = boundary.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);

    let mut points = Vec::new();
    let mut curr_y = min_y;
    while curr_y <= max_y {
        let mut curr_x = min_x;
        while curr_x <= max_x {
            if point_in_polygon(curr_x, curr_y, boundary) {
                let z_t = topo.first().map(|t| t.v0.z).unwrap_or(0.0);
                let z_d = design.first().map(|t| t.v0.z).unwrap_or(0.0);
                points.push(GridPointDelta {
                    x: curr_x,
                    y: curr_y,
                    z_topo: z_t,
                    z_design: z_d,
                    delta_z: z_d - z_t,
                });
            }
            curr_x += step;
        }
        curr_y += step;
    }

    points
}
