use crate::dxf::Point3D;
use crate::grid_engine::GridPointDelta;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolineSegment {
    pub level: f64,
    pub p0: Point3D,
    pub p1: Point3D,
    pub color_aci: u16,
}

pub fn generate_isolines(grid: &[GridPointDelta], _step: f64, levels: Vec<f64>) -> Vec<IsolineSegment> {
    let mut segments = Vec::new();

    for level in levels {
        for window in grid.windows(2) {
            let p0 = &window[0];
            let p1 = &window[1];

            if (p0.delta_z <= level && p1.delta_z >= level) || (p0.delta_z >= level && p1.delta_z <= level) {
                let color_aci = if level > 0.0 { 140 } else if level < 0.0 { 1 } else { 2 };
                segments.push(IsolineSegment {
                    level,
                    p0: Point3D { x: p0.x, y: p0.y, z: p0.delta_z },
                    p1: Point3D { x: p1.x, y: p1.y, z: p1.delta_z },
                    color_aci,
                });
            }
        }
    }

    segments
}
