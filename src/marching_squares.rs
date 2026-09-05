use crate::dxf::Point3D;
use crate::grid_engine::GridPointDelta;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolineSegment {
    pub level: f64,
    pub p0: Point3D,
    pub p1: Point3D,
    pub color_aci: u16,
}

/// Linear interpolation between two scalar values at points (x1, y1) and (x2, y2)
fn lerp_point(
    x1: f64,
    y1: f64,
    v1: f64,
    x2: f64,
    y2: f64,
    v2: f64,
    level: f64,
) -> Point3D {
    let diff = v2 - v1;
    let t = if diff.abs() < 1e-9 {
        0.5
    } else {
        ((level - v1) / diff).clamp(0.0, 1.0)
    };

    Point3D {
        x: x1 + t * (x2 - x1),
        y: y1 + t * (y2 - y1),
        z: level,
    }
}

pub fn generate_isolines(grid: &[GridPointDelta], step: f64, levels: Vec<f64>) -> Vec<IsolineSegment> {
    if grid.is_empty() {
        return Vec::new();
    }

    let effective_step = if step > 0.0 { step } else { 1.0 };

    // Group grid points by (gx, gy) coordinate index
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    for p in grid {
        if p.x < min_x { min_x = p.x; }
        if p.y < min_y { min_y = p.y; }
    }

    let mut grid_map: HashMap<(i64, i64), f64> = HashMap::with_capacity(grid.len());
    for p in grid {
        let gx = ((p.x - min_x) / effective_step).round() as i64;
        let gy = ((p.y - min_y) / effective_step).round() as i64;
        grid_map.insert((gx, gy), p.delta_z);
    }

    let mut segments = Vec::new();

    for level in levels {
        let color_aci = if level > 0.0 {
            140 // Cut (Cyan/Greenish Blue)
        } else if level < 0.0 {
            1 // Fill (Red)
        } else {
            2 // Ongrade (Yellow/Green)
        };

        // If grid has enough structured 2D cells, run true 2D Marching Squares cell evaluation
        let mut evaluated_cells = 0;

        for (&(gx, gy), &v_bl) in &grid_map {
            // Check if full 2D quad cell exists:
            // Top-Left (gx, gy+1)      Top-Right (gx+1, gy+1)
            // Bottom-Left (gx, gy)     Bottom-Right (gx+1, gy)
            let v_br_opt = grid_map.get(&(gx + 1, gy));
            let v_tr_opt = grid_map.get(&(gx + 1, gy + 1));
            let v_tl_opt = grid_map.get(&(gx, gy + 1));

            if let (Some(&v_br), Some(&v_tr), Some(&v_tl)) = (v_br_opt, v_tr_opt, v_tl_opt) {
                evaluated_cells += 1;

                let x_left = min_x + gx as f64 * effective_step;
                let x_right = min_x + (gx + 1) as f64 * effective_step;
                let y_bot = min_y + gy as f64 * effective_step;
                let y_top = min_y + (gy + 1) as f64 * effective_step;

                // 4 corners: 0=BL, 1=BR, 2=TR, 3=TL
                let mut cell_case = 0;
                if v_bl >= level { cell_case |= 1; }
                if v_br >= level { cell_case |= 2; }
                if v_tr >= level { cell_case |= 4; }
                if v_tl >= level { cell_case |= 8; }

                // 4 edges:
                // e_bot: BL to BR
                // e_right: BR to TR
                // e_top: TL to TR
                // e_left: BL to TL
                let e_bot = || lerp_point(x_left, y_bot, v_bl, x_right, y_bot, v_br, level);
                let e_right = || lerp_point(x_right, y_bot, v_br, x_right, y_top, v_tr, level);
                let e_top = || lerp_point(x_left, y_top, v_tl, x_right, y_top, v_tr, level);
                let e_left = || lerp_point(x_left, y_bot, v_bl, x_left, y_top, v_tl, level);

                match cell_case {
                    0 | 15 => {} // All outside or all inside
                    1 => {
                        segments.push(IsolineSegment { level, p0: e_left(), p1: e_bot(), color_aci });
                    }
                    2 => {
                        segments.push(IsolineSegment { level, p0: e_bot(), p1: e_right(), color_aci });
                    }
                    3 => {
                        segments.push(IsolineSegment { level, p0: e_left(), p1: e_right(), color_aci });
                    }
                    4 => {
                        segments.push(IsolineSegment { level, p0: e_right(), p1: e_top(), color_aci });
                    }
                    5 => {
                        // Saddle point ambiguity - resolve by connecting pairs
                        segments.push(IsolineSegment { level, p0: e_left(), p1: e_top(), color_aci });
                        segments.push(IsolineSegment { level, p0: e_bot(), p1: e_right(), color_aci });
                    }
                    6 => {
                        segments.push(IsolineSegment { level, p0: e_bot(), p1: e_top(), color_aci });
                    }
                    7 => {
                        segments.push(IsolineSegment { level, p0: e_left(), p1: e_top(), color_aci });
                    }
                    8 => {
                        segments.push(IsolineSegment { level, p0: e_top(), p1: e_left(), color_aci });
                    }
                    9 => {
                        segments.push(IsolineSegment { level, p0: e_top(), p1: e_bot(), color_aci });
                    }
                    10 => {
                        // Saddle point ambiguity
                        segments.push(IsolineSegment { level, p0: e_top(), p1: e_right(), color_aci });
                        segments.push(IsolineSegment { level, p0: e_left(), p1: e_bot(), color_aci });
                    }
                    11 => {
                        segments.push(IsolineSegment { level, p0: e_top(), p1: e_right(), color_aci });
                    }
                    12 => {
                        segments.push(IsolineSegment { level, p0: e_right(), p1: e_left(), color_aci });
                    }
                    13 => {
                        segments.push(IsolineSegment { level, p0: e_right(), p1: e_bot(), color_aci });
                    }
                    14 => {
                        segments.push(IsolineSegment { level, p0: e_bot(), p1: e_left(), color_aci });
                    }
                    _ => {}
                }
            }
        }

        // Fallback for linear / single-dimension grids (e.g. 1D tests or very sparse single-row data)
        if evaluated_cells == 0 {
            for window in grid.windows(2) {
                let p0 = &window[0];
                let p1 = &window[1];

                if (p0.delta_z <= level && p1.delta_z >= level) || (p0.delta_z >= level && p1.delta_z <= level) {
                    let pt_interp = lerp_point(p0.x, p0.y, p0.delta_z, p1.x, p1.y, p1.delta_z, level);
                    segments.push(IsolineSegment {
                        level,
                        p0: pt_interp,
                        p1: pt_interp,
                        color_aci,
                    });
                }
            }
        }
    }

    segments
}
