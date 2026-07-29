use crate::grid_engine::GridPointDelta;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSummary {
    pub cut_m3: f64,
    pub fill_m3: f64,
    pub net_m3: f64,
    pub cell_area_m2: f64,
}

pub fn calculate_volume(grid: &[GridPointDelta], step: f64) -> VolumeSummary {
    let cell_area = step * step;
    let mut cut = 0.0;
    let mut fill = 0.0;

    for pt in grid {
        if pt.delta_z > 0.0 {
            fill += pt.delta_z * cell_area;
        } else if pt.delta_z < 0.0 {
            cut += pt.delta_z.abs() * cell_area;
        }
    }

    VolumeSummary {
        cut_m3: cut,
        fill_m3: fill,
        net_m3: fill - cut,
        cell_area_m2: cell_area,
    }
}
